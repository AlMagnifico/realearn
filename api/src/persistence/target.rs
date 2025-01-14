use crate::persistence::{
    OscArgument, TargetValue, VirtualControlElementCharacter, VirtualControlElementId,
};
use derive_more::Display;
use enum_iterator::IntoEnumIterator;
use enumset::EnumSet;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    enum_map::Enum,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(usize)]
pub enum LearnableTargetKind {
    TrackVolume,
    TrackPan,
    RouteVolume,
    RoutePan,
    TrackArmState,
    TrackMuteState,
    TrackSoloState,
    TrackSelectionState,
    FxOnOffState,
    FxParameterValue,
    BrowseFxPresets,
    PlayRate,
    Tempo,
    TrackAutomationMode,
    TrackMonitoringMode,
    AutomationModeOverride,
    ReaperAction,
    TransportAction,
    // Could be nice to add to the list of learnable targets
    // Seek,
    // TrackParentSendState,
    // AllTrackFxOnOffState,
    // TrackPhase,
    // TrackWidth,
    // TrackVisibility,
    // FxOnlineOfflineState,
    // LoadFxSnapshot,
    // FxVisibility,
    // RouteAutomationMode,
    // RouteMonoState,
    // RouteMuteState,
    // RoutePhase,
}

/// Which target invocations to observe, based on causality. E.g. only those not triggered by
/// ReaLearn (would pick up invocations triggered by mouse interaction with REAPER but not by
/// ReaLearn mapping control).
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    enum_map::Enum,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    derive_more::Display,
)]
#[repr(usize)]
pub enum TargetTouchCause {
    #[display(fmt = "Any")]
    #[default]
    Any,
    #[display(fmt = "Caused by ReaLearn (via mapping)")]
    Realearn,
    #[display(fmt = "Not caused by ReaLearn (e.g. via mouse)")]
    Reaper,
}

#[derive(PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum Target {
    Mouse(MouseTarget),
    LastTouched(LastTouchedTarget),
    AutomationModeOverride(AutomationModeOverrideTarget),
    ReaperAction(ReaperActionTarget),
    TransportAction(TransportActionTarget),
    AnyOn(AnyOnTarget),
    #[serde(alias = "CycleThroughTracks")]
    BrowseTracks(BrowseTracksTarget),
    Seek(SeekTarget),
    PlayRate(PlayRateTarget),
    Tempo(TempoTarget),
    GoToBookmark(GoToBookmarkTarget),
    TrackArmState(TrackArmStateTarget),
    TrackParentSendState(TrackParentSendStateTarget),
    AllTrackFxOnOffState(AllTrackFxOnOffStateTarget),
    TrackMuteState(TrackMuteStateTarget),
    TrackPeak(TrackPeakTarget),
    TrackPhase(TrackPhaseTarget),
    TrackSelectionState(TrackSelectionStateTarget),
    TrackAutomationMode(TrackAutomationModeTarget),
    TrackMonitoringMode(TrackMonitoringModeTarget),
    TrackAutomationTouchState(TrackAutomationTouchStateTarget),
    TrackPan(TrackPanTarget),
    TrackWidth(TrackWidthTarget),
    TrackVolume(TrackVolumeTarget),
    #[serde(rename = "Track")]
    TrackTool(TrackToolTarget),
    TrackVisibility(TrackVisibilityTarget),
    TrackSoloState(TrackSoloStateTarget),
    #[serde(alias = "CycleThroughFx")]
    BrowseFxChain(BrowseFxChainTarget),
    FxOnOffState(FxOnOffStateTarget),
    FxOnlineOfflineState(FxOnlineOfflineStateTarget),
    LoadFxSnapshot(LoadFxSnapshotTarget),
    #[serde(alias = "CycleThroughFxPresets")]
    BrowseFxPresets(BrowseFxPresetsTarget),
    #[serde(rename = "Fx")]
    FxTool(FxToolTarget),
    FxVisibility(FxVisibilityTarget),
    FxParameterValue(FxParameterValueTarget),
    FxParameterAutomationTouchState(FxParameterAutomationTouchStateTarget),
    RouteAutomationMode(RouteAutomationModeTarget),
    RouteMonoState(RouteMonoStateTarget),
    RouteMuteState(RouteMuteStateTarget),
    RoutePhase(RoutePhaseTarget),
    RoutePan(RoutePanTarget),
    RouteVolume(RouteVolumeTarget),
    RouteTouchState(RouteTouchStateTarget),
    #[cfg(feature = "playtime")]
    ClipTransportAction(ClipTransportActionTarget),
    #[cfg(feature = "playtime")]
    ClipColumnAction(ClipColumnTarget),
    #[cfg(feature = "playtime")]
    ClipRowAction(ClipRowTarget),
    #[cfg(feature = "playtime")]
    ClipMatrixAction(ClipMatrixTarget),
    #[cfg(feature = "playtime")]
    ClipSeek(ClipSeekTarget),
    #[cfg(feature = "playtime")]
    ClipVolume(ClipVolumeTarget),
    #[cfg(feature = "playtime")]
    ClipManagement(ClipManagementTarget),
    SendMidi(SendMidiTarget),
    SendOsc(SendOscTarget),
    Dummy(DummyTarget),
    EnableInstances(EnableInstancesTarget),
    EnableMappings(EnableMappingsTarget),
    ModifyMapping(ModifyMappingTarget),
    #[serde(alias = "LoadMappingSnapshots")]
    LoadMappingSnapshot(LoadMappingSnapshotTarget),
    TakeMappingSnapshot(TakeMappingSnapshotTarget),
    #[serde(alias = "CycleThroughGroupMappings")]
    BrowseGroupMappings(BrowseGroupMappingsTarget),
    BrowsePotFilterItems(BrowsePotFilterItemsTarget),
    #[serde(alias = "NavigateWithinPotPresets")]
    BrowsePotPresets(BrowsePotPresetsTarget),
    PreviewPotPreset(PreviewPotPresetTarget),
    LoadPotPreset(LoadPotPresetTarget),
    Virtual(VirtualTarget),
}

impl Default for Target {
    fn default() -> Self {
        Self::LastTouched(Default::default())
    }
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TargetCommons {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<TargetUnit>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TargetUnit {
    Native,
    Percent,
}

impl Default for TargetUnit {
    fn default() -> Self {
        Self::Native
    }
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MouseTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub action: MouseAction,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct LastTouchedTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_targets: Option<HashSet<LearnableTargetKind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touch_cause: Option<TargetTouchCause>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct AutomationModeOverrideTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "override")]
    pub override_value: Option<AutomationModeOverride>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct ReaperActionTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<ReaperCommand>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation: Option<ActionInvocationKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TransportActionTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub action: TransportAction,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AnyOnTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub parameter: AnyOnParameter,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct BrowseTracksTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_arrange_view: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_mixer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<BrowseTracksMode>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct SeekTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_time_selection: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_loop_points: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_regions: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_project: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_view: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seek_play: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_resolution: Option<FeedbackResolution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<SeekBehavior>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct PlayRateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TempoTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GoToBookmarkTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub bookmark: BookmarkDescriptor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_time_selection: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_loop_points: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seek_behavior: Option<SeekBehavior>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackArmStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_track_grouping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_selection_ganging: Option<bool>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackParentSendStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct AllTrackFxOnOffStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_for_feedback: Option<bool>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackMuteStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_track_grouping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_selection_ganging: Option<bool>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackPeakTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackPhaseTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_for_feedback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_track_grouping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_selection_ganging: Option<bool>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackSelectionStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_arrange_view: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_mixer: Option<bool>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrackAutomationModeTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    pub mode: AutomationMode,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrackMonitoringModeTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    pub mode: MonitoringMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_selection_ganging: Option<bool>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrackAutomationTouchStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    pub touched_parameter: TouchedTrackParameter,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackPanTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_track_grouping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_selection_ganging: Option<bool>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackWidthTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_track_grouping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_selection_ganging: Option<bool>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackVolumeTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_track_grouping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_selection_ganging: Option<bool>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackToolTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<TrackToolAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_tags: Option<Vec<String>>,
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    derive_more::Display,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(usize)]
pub enum TrackToolAction {
    #[display(fmt = "None (feedback only)")]
    DoNothing,
    #[display(fmt = "Set (as instance track)")]
    SetAsInstanceTrack,
    #[display(fmt = "Pin (as instance track)")]
    PinAsInstanceTrack,
}

impl Default for TrackToolAction {
    fn default() -> Self {
        Self::DoNothing
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum MouseAction {
    /// Mouse position on the given axis.
    ///
    /// Control:
    ///
    /// - Move by (using relative control value)
    /// - Move to (use absolute control value)
    /// - Move to absolute coordinates before clicking (2 mappings for moving, 1 for clicking)
    ///
    /// Feedback:
    ///
    /// - Reflect position of the mouse cursor
    ///
    /// Future extension possibilities:
    ///
    /// - Canvas: Relative to all screens, current screen, REAPER window or focused window
    /// - Pixel density: Take pixel density into account
    MoveTo {
        #[serde(skip_serializing_if = "Option::is_none")]
        axis: Option<Axis>,
    },
    /// Like [`Self::Move`] but prefers relative control, so the glue section will feed it
    /// with relative control values whenever possible. Could make a little difference.
    MoveBy {
        #[serde(skip_serializing_if = "Option::is_none")]
        axis: Option<Axis>,
    },
    /// Button state.
    ///
    /// Control:
    ///
    /// - Press and release a mouse button
    /// - Press a mouse button and keep it pressed (press-only filter)
    /// - Just release a mouse button (release-only filter, e.g. for manual drag control)
    ///
    /// Feedback:
    ///
    /// - Whether the button is down or up
    ///
    /// Future extension possibilities:
    ///
    /// - Click or double-click a mouse button (press and immediate release, this could be a generic
    /// "Glue" option because it could be useful for other on/off targets as well).
    PressOrRelease {
        #[serde(skip_serializing_if = "Option::is_none")]
        button: Option<MouseButton>,
    },
    /// Scroll wheel.
    ///
    /// Control:
    ///
    /// - Invoke scroll wheel
    ///
    /// Feedback: None
    Scroll {
        #[serde(skip_serializing_if = "Option::is_none")]
        axis: Option<Axis>,
    },
}

impl Default for MouseAction {
    fn default() -> Self {
        Self::MoveTo {
            axis: Default::default(),
        }
    }
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    derive_more::Display,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(usize)]
pub enum Axis {
    #[display(fmt = "X (horizontal)")]
    X,
    #[display(fmt = "Y (vertical)")]
    Y,
}

impl Default for Axis {
    fn default() -> Self {
        Self::Y
    }
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    derive_more::Display,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(usize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrackVisibilityTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    /// Made obsolete in 2.14.0-pre.8.
    #[serde(skip_serializing)]
    pub poll_for_feedback: Option<bool>,
    pub area: TrackArea,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackSoloStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<TrackExclusivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<SoloBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_track_grouping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_selection_ganging: Option<bool>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BrowseFxChainTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub chain: FxChainDescriptor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_kind: Option<FxDisplayKind>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct FxOnOffStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx: Option<FxDescriptor>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct FxOnlineOfflineStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx: Option<FxDescriptor>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct LoadFxSnapshotTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx: Option<FxDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<FxSnapshot>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct BrowseFxPresetsTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx: Option<FxDescriptor>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct FxToolTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx: Option<FxDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<FxToolAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_tags: Option<Vec<String>>,
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    derive_more::Display,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(usize)]
pub enum FxToolAction {
    #[display(fmt = "None (feedback only)")]
    DoNothing,
    #[display(fmt = "Set (as instance FX)")]
    SetAsInstanceFx,
    #[display(fmt = "Pin (as instance FX)")]
    PinAsInstanceFx,
}

impl Default for FxToolAction {
    fn default() -> Self {
        Self::DoNothing
    }
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct FxVisibilityTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx: Option<FxDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_kind: Option<FxDisplayKind>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FxParameterValueTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub parameter: FxParameterDescriptor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_for_feedback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrigger: Option<bool>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FxParameterAutomationTouchStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub parameter: FxParameterDescriptor,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RouteAutomationModeTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub route: RouteDescriptor,
    pub mode: AutomationMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_for_feedback: Option<bool>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RouteMonoStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub route: RouteDescriptor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_for_feedback: Option<bool>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RouteMuteStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub route: RouteDescriptor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_for_feedback: Option<bool>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RoutePhaseTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub route: RouteDescriptor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_for_feedback: Option<bool>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RoutePanTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub route: RouteDescriptor,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RouteVolumeTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub route: RouteDescriptor,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RouteTouchStateTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub route: RouteDescriptor,
    pub touched_parameter: TouchedRouteParameter,
}

#[cfg(feature = "playtime")]
#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipTransportActionTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub slot: ClipSlotDescriptor,
    pub action: ClipTransportAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_only_if_track_armed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_column_if_slot_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_start_timing: Option<playtime_api::persistence::ClipPlayStartTiming>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_stop_timing: Option<playtime_api::persistence::ClipPlayStopTiming>,
}

#[cfg(feature = "playtime")]
#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipColumnTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub column: ClipColumnDescriptor,
    pub action: ClipColumnAction,
}

#[cfg(feature = "playtime")]
#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipRowTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub row: ClipRowDescriptor,
    pub action: ClipRowAction,
}

#[cfg(feature = "playtime")]
#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipMatrixTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub action: ClipMatrixAction,
}

#[cfg(feature = "playtime")]
#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipSeekTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub slot: ClipSlotDescriptor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_resolution: Option<FeedbackResolution>,
}

#[cfg(feature = "playtime")]
#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipVolumeTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub slot: ClipSlotDescriptor,
}

#[cfg(feature = "playtime")]
#[derive(PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClipManagementTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    pub slot: ClipSlotDescriptor,
    pub action: ClipManagementAction,
}

#[cfg(feature = "playtime")]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum ClipManagementAction {
    ClearSlot,
    FillSlotWithSelectedItem,
    EditClip,
    CopyOrPasteClip,
    AdjustClipSectionLength(AdjustClipSectionLengthAction),
}

#[cfg(feature = "playtime")]
impl Default for ClipManagementAction {
    fn default() -> Self {
        Self::ClearSlot
    }
}

#[cfg(feature = "playtime")]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AdjustClipSectionLengthAction {
    pub factor: f64,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct SendMidiTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<MidiDestination>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct DummyTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
}

#[derive(PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct SendOscTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument: Option<OscArgument>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<OscDestination>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct EnableInstancesTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<InstanceExclusivity>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct EnableMappingsTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<MappingExclusivity>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ModifyMappingTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<String>,
    pub modification: MappingModification,
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    derive_more::Display,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(usize)]
pub enum MappingModificationKind {
    #[display(fmt = "Learn target")]
    #[default]
    LearnTarget,
    #[display(fmt = "Set target to last touched")]
    SetTargetToLastTouched,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum MappingModification {
    LearnTarget(LearnTargetMappingModification),
    SetTargetToLastTouched(SetTargetToLastTouchedMappingModification),
}

impl MappingModification {
    pub fn kind(&self) -> MappingModificationKind {
        match self {
            MappingModification::LearnTarget(_) => MappingModificationKind::LearnTarget,
            MappingModification::SetTargetToLastTouched(_) => {
                MappingModificationKind::SetTargetToLastTouched
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct LearnTargetMappingModification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_targets: Option<HashSet<LearnableTargetKind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touch_cause: Option<TargetTouchCause>,
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct SetTargetToLastTouchedMappingModification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_targets: Option<HashSet<LearnableTargetKind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub touch_cause: Option<TargetTouchCause>,
}

#[derive(PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct LoadMappingSnapshotTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_mappings_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<MappingSnapshotDescForLoad>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<TargetValue>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct TakeMappingSnapshotTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_mappings_only: Option<bool>,
    #[serde(alias = "snapshot_id")]
    pub snapshot: BackwardCompatibleMappingSnapshotDescForTake,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum BackwardCompatibleMappingSnapshotDescForTake {
    Old(String),
    New(MappingSnapshotDescForTake),
}

impl Default for BackwardCompatibleMappingSnapshotDescForTake {
    fn default() -> Self {
        Self::New(Default::default())
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum MappingSnapshotDescForLoad {
    Initial,
    ById { id: String },
}

impl MappingSnapshotDescForLoad {
    pub fn id(&self) -> Option<&str> {
        match self {
            MappingSnapshotDescForLoad::Initial => None,
            MappingSnapshotDescForLoad::ById { id } => Some(id),
        }
    }
}

impl Default for MappingSnapshotDescForLoad {
    fn default() -> Self {
        Self::Initial
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum MappingSnapshotDescForTake {
    LastLoaded,
    ById { id: String },
}

impl MappingSnapshotDescForTake {
    pub fn id(&self) -> Option<&str> {
        match self {
            MappingSnapshotDescForTake::LastLoaded => None,
            MappingSnapshotDescForTake::ById { id } => Some(id),
        }
    }
}

impl Default for MappingSnapshotDescForTake {
    fn default() -> Self {
        Self::LastLoaded
    }
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct BrowseGroupMappingsTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusivity: Option<GroupMappingExclusivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct BrowsePotFilterItemsTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_kind: Option<PotFilterKind>,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct BrowsePotPresetsTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct PreviewPotPresetTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct LoadPotPresetTarget {
    #[serde(flatten)]
    pub commons: TargetCommons,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx: Option<FxDescriptor>,
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    derive_more::Display,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    enum_map::Enum,
    enumset::EnumSetType,
)]
#[enumset(no_super_impls)]
#[repr(usize)]
pub enum PotFilterKind {
    #[display(fmt = "Database")]
    Database,
    /// Is available or not
    #[display(fmt = "Availability")]
    IsAvailable,
    /// Is supported or not
    #[display(fmt = "Support")]
    IsSupported,
    /// Is user preset or factory preset
    #[display(fmt = "Content types")]
    #[serde(alias = "NksContentType")]
    IsUser,
    /// Instrument, Effect, Loop, One Shot
    #[display(fmt = "Product types")]
    #[serde(alias = "NksProductType")]
    ProductKind,
    /// Is favorite or not
    #[display(fmt = "Favorite")]
    #[serde(alias = "NksFavorite")]
    IsFavorite,
    #[display(fmt = "Project")]
    #[serde(alias = "Project")]
    Project,
    #[display(fmt = "Product")]
    #[serde(alias = "NksBank")]
    Bank,
    #[display(fmt = "Bank")]
    #[serde(alias = "NksSubBank")]
    SubBank,
    #[display(fmt = "Type")]
    #[serde(alias = "NksCategory")]
    Category,
    #[display(fmt = "Sub type")]
    #[serde(alias = "NksSubCategory")]
    SubCategory,
    #[display(fmt = "Character")]
    #[serde(alias = "NksMode")]
    Mode,
    #[display(fmt = "Preview")]
    HasPreview,
}

impl PotFilterKind {
    /// We could also use the generated `into_enum_iter()` everywhere but IDE completion
    /// in IntelliJ Rust doesn't work for that at the time of this writing.
    pub fn enum_iter() -> impl Iterator<Item = Self> + ExactSizeIterator {
        Self::into_enum_iter()
    }

    pub fn allows_excludes(&self) -> bool {
        use PotFilterKind::*;
        matches!(self, Database | Bank | SubBank)
    }

    pub fn wants_sorting(&self) -> bool {
        use PotFilterKind::*;
        matches!(
            self,
            Database | Project | Bank | SubBank | Category | SubCategory | Mode
        )
    }

    pub fn parent(&self) -> Option<Self> {
        match self {
            PotFilterKind::SubBank => Some(PotFilterKind::Bank),
            PotFilterKind::SubCategory => Some(PotFilterKind::Category),
            _ => None,
        }
    }

    pub fn dependent_kinds(&self) -> impl Iterator<Item = PotFilterKind> {
        let dep_pos = self.dependency_position();
        Self::into_enum_iter().filter(move |k| k.dependency_position() > dep_pos)
    }

    pub fn core_kinds() -> EnumSet<PotFilterKind> {
        Self::into_enum_iter()
            .filter(|k| k.is_core_kind())
            .collect()
    }

    /// Those kinds are always supported, no matter the database.
    ///
    /// The other ones are called "advanced" kinds.
    pub fn is_core_kind(&self) -> bool {
        use PotFilterKind::*;
        matches!(
            self,
            Database | IsAvailable | IsSupported | IsUser | ProductKind | IsFavorite | HasPreview
        )
    }

    /// Filter kinds with lower dependency positions affect filter kinds with higher dependency
    /// positions.
    ///
    /// This position is used by code in order to determine whether filter items need to be
    /// recalculated. E.g. when changing the category, it means the set of possible sub categories
    /// might be affected (higher position) but the set of possible banks not (lower position).
    pub fn dependency_position(&self) -> u32 {
        use PotFilterKind::*;
        match self {
            Database | IsAvailable | IsSupported | IsUser | ProductKind | IsFavorite => 0,
            Project => 1,
            Bank => 2,
            SubBank => 3,
            Category => 4,
            SubCategory => 5,
            Mode => 6,
            HasPreview => 7,
        }
    }
}

impl Default for PotFilterKind {
    fn default() -> Self {
        Self::Database
    }
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VirtualTarget {
    pub id: VirtualControlElementId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character: Option<VirtualControlElementCharacter>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum AutomationModeOverride {
    Bypass,
    Mode { mode: AutomationMode },
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum AutomationMode {
    TrimRead,
    Read,
    Touch,
    Write,
    Latch,
    LatchPreview,
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    derive_more::Display,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(usize)]
pub enum MonitoringMode {
    #[display(fmt = "Off")]
    Off,
    #[display(fmt = "Normal")]
    Normal,
    #[display(fmt = "Tape style (off when playing)")]
    TapeStyle,
}

impl Default for MonitoringMode {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TransportAction {
    PlayStop,
    PlayPause,
    Stop,
    Pause,
    Record,
    Repeat,
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
    JsonSchema,
)]
#[repr(usize)]
pub enum ClipTransportAction {
    /// Starts or stops playback.
    ///
    /// - If slot filled, starts or stops playback.
    /// - If slot empty, has no effect.
    /// - If slot recording, has no effect or stops recording.
    #[display(fmt = "Play/stop")]
    PlayStop,
    /// Starts or pauses playback.
    ///
    /// - If slot filled, starts or pauses playback.
    /// - If slot empty, has no effect.
    /// - If slot recording, has no effect.
    #[display(fmt = "Play/pause")]
    PlayPause,
    /// Stops playback or recording.
    ///
    /// - If slot filled, stops playback.
    /// - If slot empty, has no effect.
    /// - If slot recording, stops recording.
    #[display(fmt = "Stop")]
    Stop,
    /// Pauses playback.
    ///
    /// - If slot filled, pauses playback.
    /// - If slot empty, has no effect.
    /// - If slot recording, has no effect.
    #[display(fmt = "Pause")]
    Pause,
    /// Starts or stops recording.
    ///
    /// - If slot filled, starts recording or stops playback.
    /// - If slot empty, starts recording or has no effect.
    /// - If slot recording, has no effect or stops recording.
    #[display(fmt = "Record/stop")]
    RecordStop,
    /// Starts or stops playback or recording.
    ///
    /// - If slot filled, starts or stops playback.
    /// - If slot empty, starts recording or has no effect.
    /// - If slot recording, has no effect or stops recording.
    #[display(fmt = "Record/play/stop")]
    RecordPlayStop,
    /// Changes the loop setting.
    ///
    /// - If slot is filled, sets looped on or off.
    /// - If slot empty, has no effect.
    /// - If slot recording, has no effect.
    #[display(fmt = "Looped")]
    Looped,
}

impl Default for ClipTransportAction {
    fn default() -> Self {
        Self::PlayStop
    }
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
    JsonSchema,
)]
#[repr(usize)]
pub enum ClipColumnAction {
    #[display(fmt = "Stop")]
    Stop,
}

impl Default for ClipColumnAction {
    fn default() -> Self {
        Self::Stop
    }
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
    JsonSchema,
)]
#[repr(usize)]
pub enum ClipRowAction {
    #[display(fmt = "Play")]
    PlayScene,
    #[display(fmt = "Build scene")]
    BuildScene,
    #[display(fmt = "Clear")]
    ClearScene,
    #[display(fmt = "Copy or paste")]
    CopyOrPasteScene,
}

impl Default for ClipRowAction {
    fn default() -> Self {
        Self::PlayScene
    }
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
    JsonSchema,
)]
#[repr(usize)]
pub enum ClipMatrixAction {
    #[display(fmt = "Stop")]
    Stop,
    #[display(fmt = "Undo")]
    Undo,
    #[display(fmt = "Redo")]
    Redo,
    #[display(fmt = "Build scene")]
    BuildScene,
    #[display(fmt = "Set record duration to open end")]
    SetRecordDurationToOpenEnd,
    #[display(fmt = "Set record duration to 1 bar")]
    SetRecordDurationToOneBar,
    #[display(fmt = "Set record duration to 2 bars")]
    SetRecordDurationToTwoBars,
    #[display(fmt = "Set record duration to 4 bars")]
    SetRecordDurationToFourBars,
    #[display(fmt = "Set record duration to 8 bars")]
    SetRecordDurationToEightBars,
}

impl Default for ClipMatrixAction {
    fn default() -> Self {
        Self::Stop
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum AnyOnParameter {
    TrackSolo,
    TrackMute,
    TrackArm,
    TrackSelection,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ActionInvocationKind {
    Trigger,
    #[serde(alias = "Absolute")]
    Absolute14Bit,
    Absolute7Bit,
    Relative,
}

impl Default for ActionInvocationKind {
    fn default() -> Self {
        Self::Absolute14Bit
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ReaperCommand {
    Id(u32),
    Name(String),
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "address")]
pub enum TrackDescriptor {
    This {
        #[serde(flatten)]
        commons: TrackDescriptorCommons,
    },
    Master {
        #[serde(flatten)]
        commons: TrackDescriptorCommons,
    },
    Instance {
        #[serde(flatten)]
        commons: TrackDescriptorCommons,
    },
    Selected {
        #[serde(skip_serializing_if = "Option::is_none")]
        allow_multiple: Option<bool>,
    },
    Dynamic {
        #[serde(flatten)]
        commons: TrackDescriptorCommons,
        expression: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<TrackScope>,
    },
    ById {
        #[serde(flatten)]
        commons: TrackDescriptorCommons,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    ByIndex {
        #[serde(flatten)]
        commons: TrackDescriptorCommons,
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<TrackScope>,
    },
    ByName {
        #[serde(flatten)]
        commons: TrackDescriptorCommons,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        allow_multiple: Option<bool>,
    },
    #[cfg(feature = "playtime")]
    FromClipColumn {
        #[serde(flatten)]
        commons: TrackDescriptorCommons,
        column: ClipColumnDescriptor,
        context: ClipColumnTrackContext,
    },
}

#[cfg(feature = "playtime")]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
pub enum ClipColumnTrackContext {
    Playback,
    Recording,
}

#[cfg(feature = "playtime")]
impl Default for ClipColumnTrackContext {
    fn default() -> Self {
        Self::Playback
    }
}

impl Default for TrackDescriptor {
    fn default() -> Self {
        Self::This {
            commons: Default::default(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct TrackDescriptorCommons {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_must_be_selected: Option<bool>,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum FeedbackResolution {
    Beat,
    High,
}

impl Default for FeedbackResolution {
    fn default() -> Self {
        Self::Beat
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::enum_variant_names)]
pub enum TrackExclusivity {
    WithinProject,
    WithinFolder,
    WithinProjectOnOnly,
    WithinFolderOnOnly,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum InstanceExclusivity {
    Exclusive,
    ExclusiveOnOnly,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum MappingExclusivity {
    Exclusive,
    ExclusiveOnOnly,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum GroupMappingExclusivity {
    Exclusive,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TouchedTrackParameter {
    Volume,
    Pan,
    Width,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TouchedRouteParameter {
    Volume,
    Pan,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TrackArea {
    Tcp,
    Mcp,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub enum SoloBehavior {
    #[default]
    InPlace,
    IgnoreRouting,
    ReaperPreference,
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Default,
    Serialize,
    Deserialize,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
    JsonSchema,
)]
#[repr(usize)]
pub enum SeekBehavior {
    #[default]
    #[display(fmt = "Immediate")]
    Immediate,
    #[display(fmt = "Smooth")]
    Smooth,
    #[display(fmt = "Use REAPER preference")]
    ReaperPreference,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum BookmarkDescriptor {
    Marker(BookmarkRef),
    Region(BookmarkRef),
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum BookmarkRef {
    ById { id: u32 },
    ByIndex { index: u32 },
}

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct FxDescriptorCommons {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx_must_have_focus: Option<bool>,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "address")]
pub enum FxDescriptor {
    This {
        #[serde(flatten)]
        commons: FxDescriptorCommons,
    },
    Focused,
    Instance {
        #[serde(flatten)]
        commons: FxDescriptorCommons,
    },
    Dynamic {
        #[serde(flatten)]
        commons: FxDescriptorCommons,
        chain: FxChainDescriptor,
        expression: String,
    },
    ById {
        #[serde(flatten)]
        commons: FxDescriptorCommons,
        chain: FxChainDescriptor,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
    ByIndex {
        #[serde(flatten)]
        commons: FxDescriptorCommons,
        chain: FxChainDescriptor,
        index: u32,
    },
    ByName {
        #[serde(flatten)]
        commons: FxDescriptorCommons,
        chain: FxChainDescriptor,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        allow_multiple: Option<bool>,
    },
}

impl Default for FxDescriptor {
    fn default() -> Self {
        Self::This {
            commons: Default::default(),
        }
    }
}

// The best default for this would be a <This> FX chain but we don't have this yet!
// Therefore we don't implement Default at all for now. We can still do it later.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "address")]
pub enum FxChainDescriptor {
    Track {
        #[serde(skip_serializing_if = "Option::is_none")]
        track: Option<TrackDescriptor>,
        #[serde(skip_serializing_if = "Option::is_none")]
        chain: Option<TrackFxChain>,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
pub enum TrackFxChain {
    Normal,
    Input,
}

impl TrackFxChain {
    pub fn is_input_fx(&self) -> bool {
        matches!(self, Self::Input)
    }
}

impl Default for TrackFxChain {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum FxDisplayKind {
    FloatingWindow,
    Chain,
}

impl Default for FxDisplayKind {
    fn default() -> Self {
        Self::FloatingWindow
    }
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FxSnapshot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fx_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset_name: Option<String>,
    pub content: FxSnapshotContent,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum FxSnapshotContent {
    Chunk { chunk: String },
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "address")]
pub enum FxParameterDescriptor {
    Dynamic {
        #[serde(skip_serializing_if = "Option::is_none")]
        fx: Option<FxDescriptor>,
        expression: String,
    },
    ById {
        #[serde(skip_serializing_if = "Option::is_none")]
        fx: Option<FxDescriptor>,
        index: u32,
    },
    ByIndex {
        #[serde(skip_serializing_if = "Option::is_none")]
        fx: Option<FxDescriptor>,
        index: u32,
    },
    ByName {
        #[serde(skip_serializing_if = "Option::is_none")]
        fx: Option<FxDescriptor>,
        name: String,
    },
}

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
pub struct RouteDescriptorCommons {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "kind")]
    pub route_kind: Option<TrackRouteKind>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "address")]
pub enum RouteDescriptor {
    Dynamic {
        #[serde(flatten)]
        commons: RouteDescriptorCommons,
        expression: String,
    },
    ById {
        #[serde(flatten)]
        commons: RouteDescriptorCommons,
        id: Option<String>,
    },
    ByIndex {
        #[serde(flatten)]
        commons: RouteDescriptorCommons,
        index: u32,
    },
    ByName {
        #[serde(flatten)]
        commons: RouteDescriptorCommons,
        name: String,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum TrackRouteKind {
    Send,
    Receive,
    HardwareOutput,
}

impl Default for TrackRouteKind {
    fn default() -> Self {
        Self::Send
    }
}

#[cfg(feature = "playtime")]
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "address")]
pub enum ClipSlotDescriptor {
    Selected,
    ByIndex {
        column_index: usize,
        row_index: usize,
    },
    Dynamic {
        column_expression: String,
        row_expression: String,
    },
}

#[cfg(feature = "playtime")]
impl Default for ClipSlotDescriptor {
    fn default() -> Self {
        Self::Selected
    }
}

#[cfg(feature = "playtime")]
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "address")]
pub enum ClipColumnDescriptor {
    Selected,
    ByIndex { index: usize },
    Dynamic { expression: String },
}

#[cfg(feature = "playtime")]
impl Default for ClipColumnDescriptor {
    fn default() -> Self {
        Self::Selected
    }
}

#[cfg(feature = "playtime")]
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "address")]
pub enum ClipRowDescriptor {
    Selected,
    ByIndex { index: usize },
    Dynamic { expression: String },
}

#[cfg(feature = "playtime")]
impl Default for ClipRowDescriptor {
    fn default() -> Self {
        Self::Selected
    }
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum MidiDestination {
    FxOutput,
    FeedbackOutput,
}

impl Default for MidiDestination {
    fn default() -> Self {
        Self::FeedbackOutput
    }
}

#[derive(Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind")]
pub enum OscDestination {
    FeedbackOutput,
    Device { id: String },
}

impl Default for OscDestination {
    fn default() -> Self {
        Self::FeedbackOutput
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize, JsonSchema)]
#[repr(usize)]
pub enum TrackScope {
    AllTracks,
    TracksVisibleInTcp,
    TracksVisibleInMcp,
}

impl Default for TrackScope {
    fn default() -> Self {
        Self::AllTracks
    }
}

#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    derive_more::Display,
    enum_iterator::IntoEnumIterator,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
)]
#[repr(usize)]
pub enum BrowseTracksMode {
    #[display(fmt = "All tracks")]
    AllTracks,
    #[display(fmt = "Only tracks visible in TCP")]
    TracksVisibleInTcp,
    #[display(fmt = "Only tracks visible in TCP (allow 2 selections)")]
    TracksVisibleInTcpAllowTwoSelections,
    #[display(fmt = "Only tracks visible in MCP")]
    TracksVisibleInMcp,
    #[display(fmt = "Only tracks visible in MCP (allow 2 selections)")]
    TracksVisibleInMcpAllowTwoSelections,
}

impl Default for BrowseTracksMode {
    fn default() -> Self {
        Self::AllTracks
    }
}

impl BrowseTracksMode {
    pub fn scope(&self) -> TrackScope {
        use BrowseTracksMode::*;
        match self {
            AllTracks => TrackScope::AllTracks,
            TracksVisibleInTcp | TracksVisibleInTcpAllowTwoSelections => {
                TrackScope::TracksVisibleInTcp
            }
            TracksVisibleInMcp | TracksVisibleInMcpAllowTwoSelections => {
                TrackScope::TracksVisibleInMcp
            }
        }
    }
}
