use crate::domain::{MainProcessorTargetUpdate, Mode, ReaperTarget};
use helgoboss_learn::{ControlValue, MidiSource, MidiSourceValue, Target};
use helgoboss_midi::RawShortMessage;

use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug)]
pub struct ProcessorMapping {
    id: MappingId,
    source: MidiSource,
    mode: Mode,
    target: Option<ReaperTarget>,
    control_is_enabled: bool,
    feedback_is_enabled: bool,
    prevent_echo_feedback: bool,
}

impl ProcessorMapping {
    pub fn new(
        id: MappingId,
        source: MidiSource,
        mode: Mode,
        target: Option<ReaperTarget>,
        control_is_enabled: bool,
        feedback_is_enabled: bool,
        prevent_echo_feedback: bool,
    ) -> ProcessorMapping {
        ProcessorMapping {
            id,
            source,
            mode,
            target,
            control_is_enabled,
            feedback_is_enabled,
            prevent_echo_feedback,
        }
    }

    pub fn splinter(
        &self,
        feedback_is_globally_enabled: bool,
    ) -> (RealTimeProcessorMapping, MainProcessorMapping) {
        let real_time_mapping =
            RealTimeProcessorMapping::new(self.id, self.source.clone(), self.control_is_enabled);
        let main_mapping = MainProcessorMapping::new(
            self.id,
            self.source.clone(),
            self.mode.clone(),
            self.target.clone(),
            self.control_is_enabled,
            self.feedback_is_enabled && feedback_is_globally_enabled,
            self.prevent_echo_feedback,
        );
        (real_time_mapping, main_mapping)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MappingId {
    uuid: Uuid,
}

impl MappingId {
    pub fn random() -> MappingId {
        MappingId {
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
pub struct RealTimeProcessorMapping {
    id: MappingId,
    source: MidiSource,
    control_is_enabled: bool,
}

impl RealTimeProcessorMapping {
    pub fn new(
        mapping_id: MappingId,
        source: MidiSource,
        control_is_enabled: bool,
    ) -> RealTimeProcessorMapping {
        RealTimeProcessorMapping {
            source,
            id: mapping_id,
            control_is_enabled,
        }
    }

    pub fn id(&self) -> MappingId {
        self.id
    }

    pub fn control(&self, value: &MidiSourceValue<RawShortMessage>) -> Option<ControlValue> {
        self.source.control(value)
    }

    pub fn consumes(&self, msg: RawShortMessage) -> bool {
        self.source.consumes(&msg)
    }

    pub fn control_is_enabled(&self) -> bool {
        self.control_is_enabled
    }

    pub fn enable_control(&mut self) {
        self.control_is_enabled = true;
    }

    pub fn disable_control(&mut self) {
        self.control_is_enabled = false;
    }
}

const MAX_ECHO_FEEDBACK_DELAY: Duration = Duration::from_millis(20);

#[derive(Debug)]
pub struct MainProcessorMapping {
    id: MappingId,
    source: MidiSource,
    mode: Mode,
    target: Option<ReaperTarget>,
    control_is_enabled: bool,
    feedback_is_enabled: bool,
    prevent_echo_feedback: bool,
    time_of_last_control: Option<Instant>,
}

impl MainProcessorMapping {
    pub fn new(
        id: MappingId,
        source: MidiSource,
        mode: Mode,
        target: Option<ReaperTarget>,
        control_is_enabled: bool,
        feedback_is_enabled: bool,
        prevent_echo_feedback: bool,
    ) -> MainProcessorMapping {
        MainProcessorMapping {
            id,
            source,
            mode,
            target,
            control_is_enabled,
            feedback_is_enabled,
            prevent_echo_feedback,
            time_of_last_control: None,
        }
    }

    pub fn id(&self) -> MappingId {
        self.id
    }

    pub fn update_from_target(&mut self, update: MainProcessorTargetUpdate) {
        self.target = update.target;
        self.control_is_enabled = update.control_is_enabled;
        self.feedback_is_enabled = update.feedback_is_enabled;
    }

    pub fn into_main_processor_target_update(self) -> MainProcessorTargetUpdate {
        MainProcessorTargetUpdate {
            id: self.id(),
            target: self.target,
            control_is_enabled: self.control_is_enabled,
            feedback_is_enabled: self.feedback_is_enabled,
        }
    }

    pub fn control_is_enabled(&self) -> bool {
        self.control_is_enabled
    }

    pub fn feedback_is_enabled(&self) -> bool {
        self.feedback_is_enabled
    }

    pub fn control_if_enabled(&mut self, value: ControlValue) {
        if !self.control_is_enabled {
            return;
        }
        let target = match &self.target {
            None => return,
            Some(t) => t,
        };
        if let Some(final_value) = self.mode.control(value, target) {
            if self.prevent_echo_feedback {
                self.time_of_last_control = Some(Instant::now());
            }
            target.control(final_value).unwrap();
        }
    }

    pub fn feedback_if_enabled(&self) -> Option<MidiSourceValue<RawShortMessage>> {
        if !self.feedback_is_enabled {
            return None;
        }
        if let Some(t) = self.time_of_last_control {
            if t.elapsed() <= MAX_ECHO_FEEDBACK_DELAY {
                return None;
            }
        }
        let target = match &self.target {
            None => return None,
            Some(t) => t,
        };
        let target_value = target.current_value();
        let modified_value = self.mode.feedback(target_value);
        self.source.feedback(modified_value)
    }

    pub fn source(&self) -> &MidiSource {
        &self.source
    }

    pub fn target(&self) -> Option<&ReaperTarget> {
        self.target.as_ref()
    }
}
