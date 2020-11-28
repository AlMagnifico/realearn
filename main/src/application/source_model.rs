use crate::core::{prop, Prop};
use crate::domain::{
    CompoundMappingSource, ExtendedSourceCharacter, VirtualControlElement, VirtualSource,
    VirtualTarget,
};
use derive_more::Display;
use enum_iterator::IntoEnumIterator;
use helgoboss_learn::{
    ControlValue, MidiClockTransportMessage, MidiSource, SourceCharacter, UnitValue,
};
use helgoboss_midi::{Channel, U14, U7};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rx_util::UnitEvent;
use serde::export::Formatter;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::borrow::Cow;
use std::fmt::Display;

/// A model for creating sources
#[derive(Clone, Debug)]
pub struct SourceModel {
    pub category: Prop<SourceCategory>,
    pub midi_source_type: Prop<MidiSourceType>,
    pub control_element_type: Prop<VirtualControlElementType>,
    pub control_element_index: Prop<u32>,
    pub channel: Prop<Option<Channel>>,
    pub midi_message_number: Prop<Option<U7>>,
    pub parameter_number_message_number: Prop<Option<U14>>,
    pub custom_character: Prop<SourceCharacter>,
    pub midi_clock_transport_message: Prop<MidiClockTransportMessage>,
    pub is_registered: Prop<Option<bool>>,
    pub is_14_bit: Prop<Option<bool>>,
}

impl Default for SourceModel {
    fn default() -> Self {
        Self {
            category: prop(SourceCategory::Midi),
            midi_source_type: prop(MidiSourceType::ControlChangeValue),
            control_element_type: prop(VirtualControlElementType::Multi),
            control_element_index: prop(0),
            channel: prop(None),
            midi_message_number: prop(None),
            parameter_number_message_number: prop(None),
            custom_character: prop(SourceCharacter::Range),
            midi_clock_transport_message: prop(MidiClockTransportMessage::Start),
            is_registered: prop(Some(false)),
            is_14_bit: prop(Some(false)),
        }
    }
}

impl SourceModel {
    /// Fires whenever one of the properties of this model has changed
    pub fn changed(&self) -> impl UnitEvent {
        self.category
            .changed()
            .merge(self.midi_source_type.changed())
            .merge(self.channel.changed())
            .merge(self.midi_message_number.changed())
            .merge(self.parameter_number_message_number.changed())
            .merge(self.custom_character.changed())
            .merge(self.midi_clock_transport_message.changed())
            .merge(self.is_registered.changed())
            .merge(self.is_14_bit.changed())
            .merge(self.control_element_type.changed())
            .merge(self.control_element_index.changed())
    }

    pub fn apply_from_source(&mut self, source: &CompoundMappingSource) {
        use CompoundMappingSource::*;
        match source {
            Midi(s) => {
                self.category.set(SourceCategory::Midi);
                self.midi_source_type.set(MidiSourceType::from_source(s));
                self.channel.set(s.channel());
                use MidiSource::*;
                match s {
                    NoteVelocity { key_number, .. }
                    | PolyphonicKeyPressureAmount { key_number, .. } => {
                        self.midi_message_number.set(key_number.map(Into::into));
                    }
                    ControlChangeValue {
                        controller_number,
                        custom_character,
                        ..
                    } => {
                        self.is_14_bit.set(Some(false));
                        self.midi_message_number
                            .set(controller_number.map(Into::into));
                        self.custom_character.set(*custom_character);
                    }
                    ControlChange14BitValue {
                        msb_controller_number,
                        ..
                    } => {
                        self.is_14_bit.set(Some(true));
                        self.midi_message_number
                            .set(msb_controller_number.map(Into::into));
                    }
                    ParameterNumberValue {
                        number,
                        is_14_bit,
                        is_registered,
                        ..
                    } => {
                        self.parameter_number_message_number.set(*number);
                        self.is_14_bit.set(*is_14_bit);
                        self.is_registered.set(*is_registered);
                    }
                    ClockTransport { message } => {
                        self.midi_clock_transport_message.set(*message);
                    }
                    _ => {}
                }
            }
            Virtual(s) => {
                self.category.set(SourceCategory::Virtual);
                self.control_element_type
                    .set(VirtualControlElementType::from_source(s));
                self.control_element_index.set(s.control_element().index())
            }
        };
    }

    pub fn format_control_value(&self, value: ControlValue) -> Result<String, &'static str> {
        // TODO-low use cached
        self.create_source().format_control_value(value)
    }

    pub fn parse_control_value(&self, text: &str) -> Result<UnitValue, &'static str> {
        // TODO-low use cached
        self.create_source().parse_control_value(text)
    }

    pub fn character(&self) -> ExtendedSourceCharacter {
        // TODO-low use cached
        self.create_source().character()
    }

    /// Creates a source reflecting this model's current values
    pub fn create_source(&self) -> CompoundMappingSource {
        use SourceCategory::*;
        match self.category.get() {
            Midi => {
                use MidiSourceType::*;
                let channel = self.channel.get();
                let key_number = self.midi_message_number.get().map(|n| n.into());
                let midi_source = match self.midi_source_type.get() {
                    NoteVelocity => MidiSource::NoteVelocity {
                        channel,
                        key_number,
                    },
                    NoteKeyNumber => MidiSource::NoteKeyNumber { channel },
                    PolyphonicKeyPressureAmount => MidiSource::PolyphonicKeyPressureAmount {
                        channel,
                        key_number,
                    },
                    ControlChangeValue => {
                        if self.is_14_bit.get() == Some(true) {
                            MidiSource::ControlChange14BitValue {
                                channel,
                                msb_controller_number: self
                                    .midi_message_number
                                    .get()
                                    .map(|n| n.into()),
                            }
                        } else {
                            MidiSource::ControlChangeValue {
                                channel,
                                controller_number: self.midi_message_number.get().map(|n| n.into()),
                                custom_character: self.custom_character.get(),
                            }
                        }
                    }
                    ProgramChangeNumber => MidiSource::ProgramChangeNumber { channel },
                    ChannelPressureAmount => MidiSource::ChannelPressureAmount { channel },
                    PitchBendChangeValue => MidiSource::PitchBendChangeValue { channel },
                    ParameterNumberValue => MidiSource::ParameterNumberValue {
                        channel,
                        number: self.parameter_number_message_number.get(),
                        is_14_bit: self.is_14_bit.get(),
                        is_registered: self.is_registered.get(),
                    },
                    ClockTempo => MidiSource::ClockTempo,
                    ClockTransport => MidiSource::ClockTransport {
                        message: self.midi_clock_transport_message.get(),
                    },
                };
                CompoundMappingSource::Midi(midi_source)
            }
            Virtual => {
                let virtual_source = VirtualSource::new(self.create_control_element());
                CompoundMappingSource::Virtual(virtual_source)
            }
        }
    }

    pub fn supports_virtual_control_element_index(&self) -> bool {
        self.category.get() == SourceCategory::Virtual
    }

    pub fn supports_channel(&self) -> bool {
        if !self.is_midi() {
            return false;
        }
        use MidiSourceType::*;
        matches!(
            self.midi_source_type.get(),
            ChannelPressureAmount
                | ControlChangeValue
                | NoteVelocity
                | PolyphonicKeyPressureAmount
                | NoteKeyNumber
                | ParameterNumberValue
                | PitchBendChangeValue
                | ProgramChangeNumber
        )
    }

    pub fn supports_midi_message_number(&self) -> bool {
        if !self.is_midi() {
            return false;
        }
        use MidiSourceType::*;
        matches!(
            self.midi_source_type.get(),
            ControlChangeValue | NoteVelocity | PolyphonicKeyPressureAmount
        )
    }

    pub fn supports_14_bit(&self) -> bool {
        if !self.is_midi() {
            return false;
        }
        use MidiSourceType::*;
        matches!(
            self.midi_source_type.get(),
            ControlChangeValue | ParameterNumberValue
        )
    }

    pub fn supports_parameter_number_message_number(&self) -> bool {
        if !self.is_midi() {
            return false;
        }
        self.supports_parameter_number_message_props()
    }

    pub fn supports_is_registered(&self) -> bool {
        if !self.is_midi() {
            return false;
        }
        self.supports_parameter_number_message_props()
    }

    pub fn supports_custom_character(&self) -> bool {
        if !self.is_midi() {
            return false;
        }
        self.midi_source_type.get() == MidiSourceType::ControlChangeValue
            && self.is_14_bit.get().contains(&false)
    }

    pub fn supports_midi_clock_transport_message_type(&self) -> bool {
        if !self.is_midi() {
            return false;
        }
        self.midi_source_type.get() == MidiSourceType::ClockTransport
    }

    fn is_midi(&self) -> bool {
        self.category.get() == SourceCategory::Midi
    }

    fn supports_parameter_number_message_props(&self) -> bool {
        self.midi_source_type.get() == MidiSourceType::ParameterNumberValue
    }

    fn primary_label(&self) -> Cow<str> {
        use SourceCategory::*;
        match self.category.get() {
            Midi => {
                use MidiSourceType::*;
                match self.midi_source_type.get() {
                    ParameterNumberValue => match self.is_registered.get() {
                        None => ParameterNumberValue.to_string().into(),
                        Some(is_registered) => {
                            if is_registered {
                                "RPN".into()
                            } else {
                                "NRPN".into()
                            }
                        }
                    },
                    PolyphonicKeyPressureAmount => "Poly after touch".into(),
                    ClockTempo => "MIDI clock\nTempo".into(),
                    ClockTransport => {
                        format!("MIDI clock\n{}", self.midi_clock_transport_message.get()).into()
                    }
                    t => t.to_string().into(),
                }
            }
            Virtual => "Virtual".into(),
        }
    }

    fn secondary_label(&self) -> Cow<str> {
        use SourceCategory::*;
        match self.category.get() {
            Midi => {
                if self.supports_channel() {
                    match self.channel.get() {
                        None => "Any channel".into(),
                        Some(ch) => format!("Channel {}", ch.get() + 1).into(),
                    }
                } else {
                    "".into()
                }
            }
            Virtual => self.create_control_element().to_string().into(),
        }
    }

    pub fn create_control_element(&self) -> VirtualControlElement {
        self.control_element_type
            .get()
            .create_control_element(self.control_element_index.get())
    }

    fn ternary_label(&self) -> Cow<str> {
        use SourceCategory::*;
        match self.category.get() {
            Midi => {
                use MidiSourceType::*;
                match self.midi_source_type.get() {
                    NoteVelocity | PolyphonicKeyPressureAmount => {
                        match self.midi_message_number.get() {
                            None => "Any note".into(),
                            Some(n) => format!("Note number {}", n.get()).into(),
                        }
                    }
                    ControlChangeValue => match self.midi_message_number.get() {
                        None => "Any CC".into(),
                        Some(n) => format!("CC number {}", n.get()).into(),
                    },
                    ParameterNumberValue => match self.parameter_number_message_number.get() {
                        None => "Any number".into(),
                        Some(n) => format!("Number {}", n.get()).into(),
                    },
                    _ => "".into(),
                }
            }
            Virtual => "".into(),
        }
    }
}

impl Display for SourceModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let compartments = [
            self.primary_label(),
            self.secondary_label(),
            self.ternary_label(),
        ];
        write!(f, "{}", compartments.join("\n"))
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
)]
#[repr(usize)]
pub enum SourceCategory {
    #[serde(rename = "midi")]
    #[display(fmt = "MIDI")]
    Midi,
    #[serde(rename = "virtual")]
    #[display(fmt = "Virtual")]
    Virtual,
}

impl Default for SourceCategory {
    fn default() -> Self {
        SourceCategory::Midi
    }
}

/// Type of a MIDI source
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize_repr,
    Deserialize_repr,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
)]
#[repr(usize)]
pub enum MidiSourceType {
    #[display(fmt = "CC value")]
    ControlChangeValue = 0,
    #[display(fmt = "Note velocity")]
    NoteVelocity = 1,
    #[display(fmt = "Note number")]
    NoteKeyNumber = 2,
    #[display(fmt = "Pitch wheel")]
    PitchBendChangeValue = 3,
    #[display(fmt = "Channel after touch")]
    ChannelPressureAmount = 4,
    #[display(fmt = "Program change")]
    ProgramChangeNumber = 5,
    #[display(fmt = "(N)RPN value (no feedback)")]
    ParameterNumberValue = 6,
    #[display(fmt = "Polyphonic after touch")]
    PolyphonicKeyPressureAmount = 7,
    #[display(fmt = "MIDI clock tempo (experimental)")]
    ClockTempo = 8,
    #[display(fmt = "MIDI clock transport")]
    ClockTransport = 9,
}

impl Default for MidiSourceType {
    fn default() -> Self {
        MidiSourceType::ControlChangeValue
    }
}

impl MidiSourceType {
    pub fn from_source(source: &MidiSource) -> MidiSourceType {
        use MidiSource::*;
        match source {
            NoteVelocity { .. } => MidiSourceType::NoteVelocity,
            NoteKeyNumber { .. } => MidiSourceType::NoteKeyNumber,
            PolyphonicKeyPressureAmount { .. } => MidiSourceType::PolyphonicKeyPressureAmount,
            ControlChangeValue { .. } => MidiSourceType::ControlChangeValue,
            ProgramChangeNumber { .. } => MidiSourceType::ProgramChangeNumber,
            ChannelPressureAmount { .. } => MidiSourceType::ChannelPressureAmount,
            PitchBendChangeValue { .. } => MidiSourceType::PitchBendChangeValue,
            ControlChange14BitValue { .. } => MidiSourceType::ControlChangeValue,
            ParameterNumberValue { .. } => MidiSourceType::ParameterNumberValue,
            ClockTempo => MidiSourceType::ClockTempo,
            ClockTransport { .. } => MidiSourceType::ClockTransport,
        }
    }

    pub fn number_label(&self) -> &'static str {
        use MidiSourceType::*;
        match self {
            ControlChangeValue => "CC number",
            NoteVelocity | PolyphonicKeyPressureAmount => "Note number",
            ParameterNumberValue => "Number",
            _ => "",
        }
    }
}

/// Type of a virtual source
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    IntoEnumIterator,
    TryFromPrimitive,
    IntoPrimitive,
    Display,
)]
#[repr(usize)]
pub enum VirtualControlElementType {
    // TODO-high rename to multi
    #[serde(rename = "continuous")]
    #[display(fmt = "Multi")]
    Multi,
    #[serde(rename = "button")]
    #[display(fmt = "Button")]
    Button,
}

impl Default for VirtualControlElementType {
    fn default() -> Self {
        VirtualControlElementType::Multi
    }
}

impl VirtualControlElementType {
    pub fn from_source(source: &VirtualSource) -> VirtualControlElementType {
        use VirtualControlElement::*;
        match source.control_element() {
            Multi(_) => VirtualControlElementType::Multi,
            Button(_) => VirtualControlElementType::Button,
        }
    }

    pub fn from_target(target: &VirtualTarget) -> VirtualControlElementType {
        use VirtualControlElement::*;
        match target.control_element() {
            Multi(_) => VirtualControlElementType::Multi,
            Button(_) => VirtualControlElementType::Button,
        }
    }

    pub fn create_control_element(self, index: u32) -> VirtualControlElement {
        use VirtualControlElementType::*;
        match self {
            Multi => VirtualControlElement::Multi(index),
            Button => VirtualControlElement::Button(index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use helgoboss_midi::test_util::*;
    use rx_util::create_invocation_mock;
    use rxrust::prelude::*;

    #[test]
    fn changed() {
        // Given
        let mut m = SourceModel::default();
        let (mock, mock_mirror) = create_invocation_mock();
        // When
        m.changed().subscribe(move |_| mock.invoke(()));
        m.midi_source_type.set(MidiSourceType::NoteVelocity);
        m.channel.set(Some(channel(5)));
        m.midi_source_type.set(MidiSourceType::ClockTransport);
        m.midi_source_type.set(MidiSourceType::ClockTransport);
        m.channel.set(Some(channel(4)));
        // Then
        assert_eq!(mock_mirror.invocation_count(), 4);
    }

    #[test]
    fn create_source() {
        // Given
        let m = SourceModel::default();
        // When
        let s = m.create_source();
        // Then
        assert_eq!(
            s,
            CompoundMappingSource::Midi(MidiSource::ControlChangeValue {
                channel: None,
                controller_number: None,
                custom_character: SourceCharacter::Range,
            })
        );
    }
}
