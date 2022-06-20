use crate::domain::{
    Compartment, CompoundMappingTarget, MappingId, MessageCaptureResult, PluginParamIndex,
    PluginParams, ProjectionFeedbackValue, QualifiedMappingId, RawParamValue, RealearnClipMatrix,
};
use helgoboss_learn::AbsoluteValue;
use playtime_clip_engine::main::ClipMatrixEvent;
use reaper_high::{ChangeEvent, Guid};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;

/// An event which is sent to upper layers and processed there
#[derive(Debug)]
pub enum DomainEvent<'a> {
    CapturedIncomingMessage(MessageCaptureEvent),
    UpdatedOnMappings(HashSet<QualifiedMappingId>),
    UpdatedSingleMappingOnState(UpdatedSingleMappingOnStateEvent),
    UpdatedSingleParameterValue {
        index: PluginParamIndex,
        value: RawParamValue,
    },
    UpdatedAllParameters(PluginParams),
    TargetValueChanged(TargetValueChangedEvent<'a>),
    ProjectionFeedback(ProjectionFeedbackValue),
    MappingMatched(MappingMatchedEvent),
    FullResyncRequested,
    MappingEnabledChangeRequested(MappingEnabledChangeRequestedEvent),
    InstanceTrackChangeRequested(InstanceTrackChangeRequestedEvent),
    InstanceFxChangeRequested(InstanceFxChangeRequestedEvent),
    ClipMatrixPolled(&'a RealearnClipMatrix, &'a [ClipMatrixEvent]),
    ControlSurfaceChangeEventForClipEngine(&'a RealearnClipMatrix, &'a ChangeEvent),
}

#[derive(Clone, Debug)]
pub struct MessageCaptureEvent {
    pub result: MessageCaptureResult,
    pub allow_virtual_sources: bool,
    pub osc_arg_index_hint: Option<u32>,
}

#[derive(Copy, Clone, Debug)]
pub struct UpdatedSingleMappingOnStateEvent {
    pub id: QualifiedMappingId,
    pub is_on: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct MappingEnabledChangeRequestedEvent {
    pub compartment: Compartment,
    pub mapping_id: MappingId,
    pub is_enabled: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum InstanceTrackChangeRequestedEvent {
    /// `None` means master track.
    Pin(Option<Guid>),
    SetFromMapping(QualifiedMappingId),
}

#[derive(Copy, Clone, Debug)]
pub enum InstanceFxChangeRequestedEvent {
    Pin {
        /// `None` means master track.
        track_guid: Option<Guid>,
        is_input_fx: bool,
        fx_guid: Guid,
    },
    SetFromMapping(QualifiedMappingId),
}

#[derive(Copy, Clone, Debug)]
pub struct MappingMatchedEvent {
    pub compartment: Compartment,
    pub mapping_id: MappingId,
}

impl MappingMatchedEvent {
    pub fn new(compartment: Compartment, mapping_id: MappingId) -> Self {
        MappingMatchedEvent {
            compartment,
            mapping_id,
        }
    }
}

#[derive(Debug)]
pub struct TargetValueChangedEvent<'a> {
    pub compartment: Compartment,
    pub mapping_id: MappingId,
    pub targets: &'a [CompoundMappingTarget],
    pub new_value: AbsoluteValue,
}

pub trait DomainEventHandler: Debug {
    fn handle_event_ignoring_error(&self, event: DomainEvent) {
        let _ = self.handle_event(event);
    }

    fn handle_event(&self, event: DomainEvent) -> Result<(), Box<dyn Error>>;

    fn notify_mapping_matched(&self, compartment: Compartment, mapping_id: MappingId) {
        self.handle_event_ignoring_error(DomainEvent::MappingMatched(MappingMatchedEvent::new(
            compartment,
            mapping_id,
        )));
    }

    /// Returns `true` if another preset is being loaded.
    fn auto_load_different_preset_if_necessary(&self) -> Result<bool, &'static str>;
}
