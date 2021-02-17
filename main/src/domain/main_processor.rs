use crate::domain::{
    CompoundMappingSource, CompoundMappingSourceValue, CompoundMappingTarget, DomainEvent,
    DomainEventHandler, FeedbackBuffer, FeedbackRealTimeTask, MainMapping, MappingActivationEffect,
    MappingActivationUpdate, MappingCompartment, MappingId, NormalRealTimeTask, ProcessorContext,
    ReaperTarget,
};
use crossbeam_channel::Sender;
use enum_iterator::IntoEnumIterator;
use enum_map::EnumMap;
use helgoboss_learn::{ControlValue, MidiSource, OscSourceValue, UnitValue};

use crate::core::Global;
use reaper_high::Reaper;
use rosc::{OscMessage, OscPacket};
use rx_util::UnitEvent;
use rxrust::prelude::*;
use slog::debug;
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};

const NORMAL_TASK_BULK_SIZE: usize = 32;
const FEEDBACK_TASK_BULK_SIZE: usize = 32;
const CONTROL_TASK_BULK_SIZE: usize = 32;
const PARAMETER_TASK_BULK_SIZE: usize = 32;

type FeedbackSubscriptionGuard = SubscriptionGuard<Box<dyn SubscriptionLike>>;
type FeedbackSubscriptions = HashMap<MappingId, FeedbackSubscriptionGuard>;

// TODO-low Making this a usize might save quite some code
pub const PLUGIN_PARAMETER_COUNT: u32 = 100;
pub type ParameterArray = [f32; PLUGIN_PARAMETER_COUNT as usize];
pub const ZEROED_PLUGIN_PARAMETERS: ParameterArray = [0.0f32; PLUGIN_PARAMETER_COUNT as usize];

#[derive(Debug)]
pub struct MainProcessor<EH: DomainEventHandler> {
    instance_id: String,
    logger: slog::Logger,
    /// Contains all mappings.
    mappings: EnumMap<MappingCompartment, HashMap<MappingId, MainMapping>>,
    /// Contains IDs of those mappings which should be refreshed as soon as a target is touched.
    /// At the moment only "Last touched" targets.
    target_touch_dependent_mappings: EnumMap<MappingCompartment, HashSet<MappingId>>,
    feedback_buffer: FeedbackBuffer,
    feedback_subscriptions: EnumMap<MappingCompartment, FeedbackSubscriptions>,
    feedback_is_globally_enabled: bool,
    self_feedback_sender: crossbeam_channel::Sender<FeedbackMainTask>,
    self_normal_sender: crossbeam_channel::Sender<NormalMainTask>,
    normal_task_receiver: crossbeam_channel::Receiver<NormalMainTask>,
    feedback_task_receiver: crossbeam_channel::Receiver<FeedbackMainTask>,
    parameter_task_receiver: crossbeam_channel::Receiver<ParameterMainTask>,
    control_task_receiver: crossbeam_channel::Receiver<ControlMainTask>,
    normal_real_time_task_sender: crossbeam_channel::Sender<NormalRealTimeTask>,
    feedback_real_time_task_sender: crossbeam_channel::Sender<FeedbackRealTimeTask>,
    parameters: ParameterArray,
    event_handler: EH,
    context: ProcessorContext,
    party_is_over_subject: LocalSubject<'static, (), ()>,
}

impl<EH: DomainEventHandler> MainProcessor<EH> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        instance_id: String,
        parent_logger: &slog::Logger,
        self_normal_sender: crossbeam_channel::Sender<NormalMainTask>,
        normal_task_receiver: crossbeam_channel::Receiver<NormalMainTask>,
        parameter_task_receiver: crossbeam_channel::Receiver<ParameterMainTask>,
        control_task_receiver: crossbeam_channel::Receiver<ControlMainTask>,
        normal_real_time_task_sender: crossbeam_channel::Sender<NormalRealTimeTask>,
        feedback_real_time_task_sender: crossbeam_channel::Sender<FeedbackRealTimeTask>,
        event_handler: EH,
        context: ProcessorContext,
    ) -> MainProcessor<EH> {
        let (self_feedback_sender, feedback_task_receiver) = crossbeam_channel::unbounded();
        let logger = parent_logger.new(slog::o!("struct" => "MainProcessor"));
        MainProcessor {
            instance_id,
            logger: logger.clone(),
            self_normal_sender,
            self_feedback_sender,
            normal_task_receiver,
            feedback_task_receiver,
            control_task_receiver,
            parameter_task_receiver,
            normal_real_time_task_sender,
            feedback_real_time_task_sender,
            mappings: Default::default(),
            target_touch_dependent_mappings: Default::default(),
            feedback_buffer: Default::default(),
            feedback_subscriptions: Default::default(),
            feedback_is_globally_enabled: false,
            parameters: ZEROED_PLUGIN_PARAMETERS,
            event_handler,
            context,
            party_is_over_subject: Default::default(),
        }
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn activate(&self) {
        // Handle dynamic target changes and target activation depending on REAPER state.
        //
        // Whenever anything changes that just affects the main processor targets, resync all
        // targets to the main processor. We don't want to resync to the real-time processor
        // just because another track has been selected. First, it would reset any source state
        // (e.g. short/long press timers). Second, it wouldn't change anything about the sources.
        // We also don't want to resync modes to the main processor. First, it would reset any
        // mode state (e.g. throttling data). Second, it would - again - not result in any change.
        // There are several global conditions which affect whether feedback will be sent
        // from a target or not. Similar global conditions decide what exactly produces the
        // feedback values (e.g. when there's a target which uses <Selected track>,
        // then a track selection change changes the feedback value producer ... so
        // the main processor needs to unsubscribe from the old producer and
        // subscribe to the new one).
        // TODO-medium We have prepared reaper-rs and ReaLearn enough to get rid of rxRust in this
        //  layer! We would just need to provide a method on ReaperTarget that takes a ChangeEvent
        //  and returns if it's affected or not.
        let self_sender = self.self_normal_sender.clone();
        ReaperTarget::potential_static_change_events()
            .merge(ReaperTarget::potential_dynamic_change_events())
            // We have this explicit stop criteria because we listen to global REAPER events.
            .take_until(self.party_is_over_subject.clone())
            .subscribe(move |_| {
                // This should always succeed because at the time this is executed, "party is not
                // yet over" and therefore the receiver exists.
                self_sender.send(NormalMainTask::RefreshAllTargets).unwrap();
            });
    }

    /// This should be regularly called by the control surface in normal mode.
    pub fn run_all(&mut self) {
        self.run_essential();
        self.run_control();
    }

    /// This should *not* be called by the control surface when it's globally learning targets
    /// because we want to pause controlling in that case! Otherwise we could control targets and
    /// they would be learned although not touched via mouse, that's not good.
    fn run_control(&mut self) {
        // Process control tasks
        let control_tasks: SmallVec<[ControlMainTask; CONTROL_TASK_BULK_SIZE]> = self
            .control_task_receiver
            .try_iter()
            .take(CONTROL_TASK_BULK_SIZE)
            .collect();
        for task in control_tasks {
            use ControlMainTask::*;
            match task {
                Control {
                    compartment,
                    mapping_id,
                    value,
                    options,
                } => {
                    if let Some(m) = self.mappings[compartment].get_mut(&mapping_id) {
                        control_and_optionally_feedback(
                            &self.feedback_real_time_task_sender,
                            m,
                            value,
                            options,
                        );
                    };
                }
            }
        }
    }

    /// This should be regularly called by the control surface, even during global target learning.
    pub fn run_essential(&mut self) {
        // Process normal tasks
        // We could also iterate directly while keeping the receiver open. But that would (for
        // good reason) prevent us from calling other methods that mutably borrow
        // self. To at least avoid heap allocations, we use a smallvec.
        let normal_tasks: SmallVec<[NormalMainTask; NORMAL_TASK_BULK_SIZE]> = self
            .normal_task_receiver
            .try_iter()
            .take(NORMAL_TASK_BULK_SIZE)
            .collect();
        let normal_task_count = normal_tasks.len();
        for task in normal_tasks {
            use NormalMainTask::*;
            match task {
                UpdateAllMappings(compartment, mappings) => {
                    debug!(
                        self.logger,
                        "Updating {} {}...",
                        mappings.len(),
                        compartment
                    );
                    let mut unused_sources = self.currently_feedback_enabled_sources(compartment);
                    // Refresh and put into hash map in order to quickly look up mappings by ID
                    self.target_touch_dependent_mappings[compartment].clear();
                    self.mappings[compartment] = mappings
                        .into_iter()
                        .map(|mut m| {
                            m.refresh_all(&self.context, &self.parameters);
                            if m.feedback_is_effectively_on() {
                                // Mark source as used
                                unused_sources.remove(m.source());
                            }
                            if m.needs_refresh_when_target_touched() {
                                self.target_touch_dependent_mappings[compartment].insert(m.id());
                            }
                            (m.id(), m)
                        })
                        .collect();
                    // Sync to real-time processor
                    let real_time_mappings = self.mappings[compartment]
                        .values()
                        .map(|m| m.splinter_real_time_mapping())
                        .collect();
                    self.normal_real_time_task_sender
                        .send(NormalRealTimeTask::UpdateAllMappings(
                            compartment,
                            real_time_mappings,
                        ))
                        .unwrap();
                    self.handle_feedback_after_batch_mapping_update(compartment, &unused_sources);
                    self.update_on_mappings();
                }
                RefreshAllTargets => {
                    debug!(self.logger, "Refreshing all targets...");
                    for compartment in MappingCompartment::into_enum_iter() {
                        let mut unused_sources =
                            self.currently_feedback_enabled_sources(compartment);
                        let mut mappings_with_active_targets =
                            HashSet::with_capacity(self.mappings[compartment].len());
                        for m in self.mappings[compartment].values_mut() {
                            let is_active = m.refresh_target(&self.context);
                            if is_active {
                                mappings_with_active_targets.insert(m.id());
                            }
                            if m.feedback_is_effectively_on() {
                                // Mark source as used
                                unused_sources.remove(m.source());
                            }
                        }
                        // In some cases like closing projects, it's possible that this will fail
                        // because the real-time processor is already gone. But it doesn't matter.
                        let _ = self.normal_real_time_task_sender.send(
                            NormalRealTimeTask::UpdateTargetActivations(
                                compartment,
                                mappings_with_active_targets,
                            ),
                        );
                        self.handle_feedback_after_batch_mapping_update(
                            compartment,
                            &unused_sources,
                        );
                    }
                    self.update_on_mappings();
                }
                UpdateSingleMapping(compartment, mut mapping) => {
                    debug!(
                        self.logger,
                        "Updating single {} {:?}...",
                        compartment,
                        mapping.id()
                    );
                    // Refresh
                    mapping.refresh_all(&self.context, &self.parameters);
                    // Sync to real-time processor
                    self.normal_real_time_task_sender
                        .send(NormalRealTimeTask::UpdateSingleMapping(
                            compartment,
                            Box::new(mapping.splinter_real_time_mapping()),
                        ))
                        .unwrap();
                    // (Re)subscribe to or unsubscribe from feedback
                    if self.feedback_is_globally_enabled {
                        match mapping.target() {
                            Some(CompoundMappingTarget::Reaper(target))
                                if mapping.feedback_is_effectively_on() =>
                            {
                                // (Re)subscribe
                                let subscription = send_feedback_when_target_value_changed(
                                    self.self_feedback_sender.clone(),
                                    compartment,
                                    mapping.id(),
                                    target,
                                );
                                self.feedback_subscriptions[compartment]
                                    .insert(mapping.id(), subscription);
                                self.send_feedback(mapping.feedback_if_enabled());
                            }
                            _ => {
                                // Unsubscribe (if the feedback was enabled before)
                                self.feedback_subscriptions[compartment].remove(&mapping.id());
                                // Indicate via feedback that this source is not in use anymore. But
                                // only if feedback was enabled before (otherwise this could
                                // overwrite the feedback value of
                                // another enabled mapping which has the same
                                // source).
                                let was_previously_enabled = self.mappings[compartment]
                                    .get(&mapping.id())
                                    .map(|m| m.feedback_is_effectively_on())
                                    .contains(&true);
                                if was_previously_enabled {
                                    // We assume that there's no other enabled mapping with the same
                                    // source at this moment. It there is, it would be a weird setup
                                    // with two conflicting feedback value sources - this wouldn't
                                    // work well anyway.
                                    self.send_feedback(mapping.source().feedback(UnitValue::MIN));
                                }
                            }
                        };
                    }
                    // Update hash map entry
                    if mapping.needs_refresh_when_target_touched() {
                        self.target_touch_dependent_mappings[compartment].insert(mapping.id());
                    } else {
                        self.target_touch_dependent_mappings[compartment].remove(&mapping.id());
                    }
                    self.mappings[compartment].insert(mapping.id(), *mapping);
                    // TODO-low Mmh, iterating over all mappings might be a bit overkill here.
                    self.update_on_mappings();
                }
                FeedbackAll => {
                    if self.feedback_is_globally_enabled {
                        self.send_feedback(self.feedback_all());
                    }
                }
                LogDebugInfo => {
                    self.log_debug_info(normal_task_count);
                }
                LearnSource {
                    source,
                    allow_virtual_sources,
                } => {
                    self.event_handler.handle_event(DomainEvent::LearnedSource {
                        source,
                        allow_virtual_sources,
                    });
                }
                UpdateFeedbackIsGloballyEnabled(is_enabled) => {
                    self.feedback_is_globally_enabled = is_enabled;
                    if is_enabled {
                        for compartment in MappingCompartment::into_enum_iter() {
                            self.handle_feedback_after_batch_mapping_update(
                                compartment,
                                &HashSet::new(),
                            );
                        }
                    } else {
                        for compartment in MappingCompartment::into_enum_iter() {
                            self.feedback_subscriptions[compartment].clear();
                        }
                        self.feedback_buffer.reset_all();
                        self.send_feedback(self.feedback_all_zero());
                    }
                }
            }
        }
        // Process parameter tasks
        let parameter_tasks: SmallVec<[ParameterMainTask; PARAMETER_TASK_BULK_SIZE]> = self
            .parameter_task_receiver
            .try_iter()
            .take(PARAMETER_TASK_BULK_SIZE)
            .collect();
        for task in parameter_tasks {
            use ParameterMainTask::*;
            match task {
                UpdateAllParameters(parameters) => {
                    debug!(self.logger, "Updating all parameters...");
                    self.parameters = *parameters;
                    // Activation is only supported for main mappings
                    let compartment = MappingCompartment::MainMappings;
                    let mut activation_updates: Vec<MappingActivationUpdate> = vec![];
                    let mut unused_sources = self.currently_feedback_enabled_sources(compartment);
                    for m in &mut self.mappings[compartment].values_mut() {
                        if m.can_be_affected_by_parameters() {
                            m.refresh_activation(&self.parameters);
                            let update = MappingActivationUpdate::new(m.id(), m.is_active());
                            activation_updates.push(update);
                        }
                        if m.feedback_is_effectively_on() {
                            // Mark source as used
                            unused_sources.remove(m.source());
                        }
                    }
                    self.process_activation_updates(
                        compartment,
                        activation_updates,
                        &unused_sources,
                    );
                }
                UpdateParameter { index, value } => {
                    debug!(self.logger, "Updating parameter {} to {}...", index, value);
                    // Workaround REAPER's inability to notify about parameter changes in monitoring
                    // FX by simulating the notification ourselves. Then parameter learning and
                    // feedback works at least for ReaLearn monitoring FX instances, which is
                    // especially useful for conditional activation.
                    if self.context.is_on_monitoring_fx_chain() {
                        let parameter = self.context.containing_fx().parameter_by_index(index);
                        let rx = Global::control_surface_rx();
                        rx.fx_parameter_value_changed
                            .borrow_mut()
                            .next(parameter.clone());
                        rx.fx_parameter_touched.borrow_mut().next(parameter);
                    }
                    // Update own value (important to do first)
                    let previous_value = self.parameters[index as usize];
                    self.parameters[index as usize] = value;
                    // Activation is only supported for main mappings
                    let compartment = MappingCompartment::MainMappings;
                    let mut unused_sources = self.currently_feedback_enabled_sources(compartment);
                    // In order to avoid a mutable borrow of mappings and an immutable borrow of
                    // parameters at the same time, we need to separate into READ activation
                    // affects and WRITE activation updates.
                    // 1. Read
                    let activation_effects: Vec<MappingActivationEffect> = self.mappings
                        [compartment]
                        .values()
                        .filter_map(|m| {
                            m.check_activation_effect(&self.parameters, index, previous_value)
                        })
                        .collect();
                    // 2. Write
                    let activation_updates: Vec<MappingActivationUpdate> = activation_effects
                        .into_iter()
                        .filter_map(|eff| {
                            let m = self.mappings[compartment].get_mut(&eff.id)?;
                            m.update_activation(eff)
                        })
                        .collect();
                    // Determine unused sources
                    for m in self.mappings[compartment].values() {
                        if m.feedback_is_effectively_on() {
                            // Mark source as used
                            unused_sources.remove(m.source());
                        }
                    }
                    self.process_activation_updates(
                        compartment,
                        activation_updates,
                        &unused_sources,
                    )
                }
            }
        }
        // Process feedback tasks
        let feedback_tasks: SmallVec<[FeedbackMainTask; FEEDBACK_TASK_BULK_SIZE]> = self
            .feedback_task_receiver
            .try_iter()
            .take(FEEDBACK_TASK_BULK_SIZE)
            .collect();
        for task in feedback_tasks {
            use FeedbackMainTask::*;
            match task {
                Feedback(compartment, mapping_id) => {
                    self.feedback_buffer
                        .buffer_feedback_for_mapping(compartment, mapping_id);
                }
                TargetTouched => {
                    for compartment in MappingCompartment::into_enum_iter() {
                        for mapping_id in self.target_touch_dependent_mappings[compartment].iter() {
                            if let Some(m) = self.mappings[compartment].get_mut(&mapping_id) {
                                m.refresh_target(&self.context);
                                // Switching off shouldn't be necessary since the last touched
                                // target can never be "unset".
                                if self.feedback_is_globally_enabled
                                    && m.feedback_is_effectively_on()
                                {
                                    if let Some(CompoundMappingTarget::Reaper(target)) = m.target()
                                    {
                                        // (Re)subscribe
                                        let subscription = send_feedback_when_target_value_changed(
                                            self.self_feedback_sender.clone(),
                                            compartment,
                                            m.id(),
                                            target,
                                        );
                                        self.feedback_subscriptions[compartment]
                                            .insert(m.id(), subscription);
                                        send_feedback(
                                            &self.feedback_real_time_task_sender,
                                            m.feedback_if_enabled(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Send feedback as soon as buffered long enough
        if self.feedback_is_globally_enabled {
            if let Some(mapping_ids) = self.feedback_buffer.poll() {
                let source_values = mapping_ids.iter().filter_map(|(compartment, mapping_id)| {
                    let mapping = self.mappings[*compartment].get(mapping_id)?;
                    mapping.feedback_if_enabled()
                });
                self.send_feedback(source_values);
            }
        }
    }

    pub fn notify_target_touched(&self) {
        self.self_feedback_sender
            .send(FeedbackMainTask::TargetTouched)
            .unwrap();
    }

    pub fn process_incoming_osc_packet(&mut self, packet: &OscPacket) {
        // TODO-high Support control_is_globally_enabled (see RealTimeProcessor)
        match packet {
            OscPacket::Message(msg) => self.process_incoming_osc_message(msg),
            // TODO-high Support bundles
            OscPacket::Bundle(_) => {}
        }
    }

    fn process_incoming_osc_message(&mut self, msg: &OscMessage) {
        let source_value = OscSourceValue::Plain(msg);
        // TODO-high Support local learning (currently handled in real-time processor only)
        // TODO-high Process virtual mappings
        let compartment = MappingCompartment::MainMappings;
        for mut m in self.mappings[compartment]
            .values_mut()
            .filter(|m| m.control_is_effectively_on())
        {
            // TODO-high Use source::control as soon as CompoundMappingSourceValue has OSC, too
            if let CompoundMappingSource::Osc(s) = m.source() {
                if let Some(control_value) = s.control(source_value) {
                    control_and_optionally_feedback(
                        &self.feedback_real_time_task_sender,
                        &mut m,
                        control_value,
                        ControlOptions {
                            enforce_send_feedback_after_control: false,
                        },
                    );
                }
            }
        }
    }

    fn process_activation_updates(
        &mut self,
        compartment: MappingCompartment,
        activation_updates: Vec<MappingActivationUpdate>,
        unused_sources: &HashSet<CompoundMappingSource>,
    ) {
        if activation_updates.is_empty() {
            return;
        }
        // Send feedback
        // TODO-low Feedback could be reduced to just the activation update mappings
        self.handle_feedback_after_batch_mapping_update(compartment, &unused_sources);
        // Communicate changes to real-time processor
        self.normal_real_time_task_sender
            .send(NormalRealTimeTask::UpdateMappingActivations(
                compartment,
                activation_updates,
            ))
            .unwrap();
        // Update on mappings
        // TODO-low Mmh, iterating over all mappings might be a bit overkill here.
        self.update_on_mappings();
    }

    fn update_on_mappings(&self) {
        let on_mappings = self
            .all_mappings()
            .filter(|m| m.is_effectively_on())
            .map(MainMapping::id)
            .collect();
        self.event_handler
            .handle_event(DomainEvent::UpdatedOnMappings(on_mappings));
    }

    fn send_feedback(&self, source_values: impl IntoIterator<Item = CompoundMappingSourceValue>) {
        send_feedback(&self.feedback_real_time_task_sender, source_values);
    }

    fn all_mappings(&self) -> impl Iterator<Item = &MainMapping> {
        MappingCompartment::into_enum_iter()
            .map(move |compartment| self.mappings[compartment].values())
            .flatten()
    }

    fn feedback_all(&self) -> Vec<CompoundMappingSourceValue> {
        self.all_mappings()
            .filter_map(|m| m.feedback_if_enabled())
            .collect()
    }

    fn feedback_all_in_compartment(
        &self,
        compartment: MappingCompartment,
    ) -> Vec<CompoundMappingSourceValue> {
        self.mappings[compartment]
            .values()
            .filter_map(|m| m.feedback_if_enabled())
            .collect()
    }

    fn feedback_all_zero(&self) -> Vec<CompoundMappingSourceValue> {
        self.all_mappings()
            .filter(|m| m.feedback_is_effectively_on())
            .filter_map(|m| m.source().feedback(UnitValue::MIN))
            .collect()
    }

    fn currently_feedback_enabled_sources(
        &self,
        compartment: MappingCompartment,
    ) -> HashSet<CompoundMappingSource> {
        self.mappings[compartment]
            .values()
            .filter(|m| m.feedback_is_effectively_on())
            .map(|m| m.source().clone())
            .collect()
    }

    fn handle_feedback_after_batch_mapping_update(
        &mut self,
        compartment: MappingCompartment,
        now_unused_sources: &HashSet<CompoundMappingSource>,
    ) {
        if !self.feedback_is_globally_enabled {
            return;
        }
        // Subscribe to target value changes for feedback. Before that, cancel all existing
        // subscriptions.
        self.feedback_subscriptions[compartment].clear();
        for m in self.mappings[compartment]
            .values()
            .filter(|m| m.feedback_is_effectively_on())
        {
            if let Some(CompoundMappingTarget::Reaper(target)) = m.target() {
                // Subscribe
                let subscription = send_feedback_when_target_value_changed(
                    self.self_feedback_sender.clone(),
                    compartment,
                    m.id(),
                    target,
                );
                self.feedback_subscriptions[compartment].insert(m.id(), subscription);
            }
        }
        // Send feedback instantly to reflect this change in mappings.
        // At first indicate via feedback the sources which are not in use anymore.
        for s in now_unused_sources {
            self.send_feedback(s.feedback(UnitValue::MIN));
        }
        // Then discard the current feedback buffer and send feedback for all new mappings which
        // are enabled.
        self.feedback_buffer.reset_all_in_compartment(compartment);
        self.send_feedback(self.feedback_all_in_compartment(compartment));
    }

    fn log_debug_info(&mut self, task_count: usize) {
        // Summary
        let msg = format!(
            "\n\
                        # Main processor\n\
                        \n\
                        - Total main mapping count: {} \n\
                        - Enabled main mapping count: {} \n\
                        - Main mapping feedback subscription count: {} \n\
                        - Total controller mapping count: {} \n\
                        - Enabled controller mapping count: {} \n\
                        - Controller mapping feedback subscription count: {} \n\
                        - Feedback buffer length: {} \n\
                        - Normal task count: {} \n\
                        - Control task count: {} \n\
                        - Feedback task count: {} \n\
                        - Parameter values: {:?} \n\
                        ",
            self.mappings[MappingCompartment::MainMappings].len(),
            self.mappings[MappingCompartment::MainMappings]
                .values()
                .filter(|m| m.control_is_effectively_on() || m.feedback_is_effectively_on())
                .count(),
            self.feedback_subscriptions[MappingCompartment::MainMappings].len(),
            self.mappings[MappingCompartment::ControllerMappings].len(),
            self.mappings[MappingCompartment::ControllerMappings]
                .values()
                .filter(|m| m.control_is_effectively_on() || m.feedback_is_effectively_on())
                .count(),
            self.feedback_subscriptions[MappingCompartment::ControllerMappings].len(),
            self.feedback_buffer.len(),
            task_count,
            self.control_task_receiver.len(),
            self.feedback_task_receiver.len(),
            self.parameters,
        );
        Reaper::get().show_console_msg(msg);
        // Detailled
        println!(
            "\n\
            # Main processor\n\
            \n\
            {:#?}
            ",
            self
        );
    }
}

fn send_feedback_when_target_value_changed(
    self_sender: Sender<FeedbackMainTask>,
    compartment: MappingCompartment,
    mapping_id: MappingId,
    target: &ReaperTarget,
) -> FeedbackSubscriptionGuard {
    target
        .value_changed()
        .subscribe(move |_| {
            self_sender
                .send(FeedbackMainTask::Feedback(compartment, mapping_id))
                .unwrap();
        })
        .unsubscribe_when_dropped()
}

/// A task which is sent from time to time.
#[derive(Debug)]
pub enum NormalMainTask {
    /// Clears all mappings and uses the passed ones.
    UpdateAllMappings(MappingCompartment, Vec<MainMapping>),
    /// Replaces the given mapping.
    // Boxed because much larger struct size than other variants.
    UpdateSingleMapping(MappingCompartment, Box<MainMapping>),
    RefreshAllTargets,
    UpdateFeedbackIsGloballyEnabled(bool),
    FeedbackAll,
    LogDebugInfo,
    LearnSource {
        source: MidiSource,
        allow_virtual_sources: bool,
    },
}

/// A parameter-related task (which is potentially sent very frequently, just think of automation).
#[derive(Debug)]
pub enum ParameterMainTask {
    UpdateParameter { index: u32, value: f32 },
    UpdateAllParameters(Box<ParameterArray>),
}

/// A feedback-related task (which is potentially sent very frequently).
#[derive(Debug)]
pub enum FeedbackMainTask {
    /// Sent whenever a target value has been changed.
    Feedback(MappingCompartment, MappingId),
    /// Sent whenever a target has been touched (usually a subset of the value change events)
    /// and as a result the global "last touched target" has been updated.
    TargetTouched,
}

/// A control-related task (which is potentially sent very frequently).
pub enum ControlMainTask {
    Control {
        compartment: MappingCompartment,
        mapping_id: MappingId,
        value: ControlValue,
        options: ControlOptions,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ControlOptions {
    pub enforce_send_feedback_after_control: bool,
}

impl<EH: DomainEventHandler> Drop for MainProcessor<EH> {
    fn drop(&mut self) {
        debug!(self.logger, "Dropping main processor...");
        self.party_is_over_subject.next(());
    }
}

fn control_and_optionally_feedback(
    sender: &crossbeam_channel::Sender<FeedbackRealTimeTask>,
    mapping: &mut MainMapping,
    value: ControlValue,
    options: ControlOptions,
) {
    // Most of the time, the main processor won't even receive a MIDI-triggered control
    // instruction from the real-time processor for a mapping for which
    // control is disabled, because the real-time processor doesn't process
    // disabled mappings. But if control is (temporarily) disabled because a
    // target condition is (temporarily) not met (e.g. "track must be
    // selected") and the real-time processor doesn't yet know about it, there
    // might be a short amount of time where we still receive control
    // statements. We filter them here.
    let feedback = mapping.control_if_enabled(value, options);
    send_feedback(sender, feedback);
}

fn send_feedback(
    sender: &crossbeam_channel::Sender<FeedbackRealTimeTask>,
    source_values: impl IntoIterator<Item = CompoundMappingSourceValue>,
) {
    for v in source_values.into_iter() {
        sender.send(FeedbackRealTimeTask::Feedback(v)).unwrap();
    }
}
