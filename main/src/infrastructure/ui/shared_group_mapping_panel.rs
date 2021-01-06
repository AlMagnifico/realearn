use crate::core::{when, Prop};
use crate::infrastructure::ui::bindings::root;
use crate::infrastructure::ui::constants::symbols;
use crate::infrastructure::ui::{MainPanel, MappingPanel};

use enum_iterator::IntoEnumIterator;
use helgoboss_learn::{
    AbsoluteMode, ControlValue, MidiClockTransportMessage, OutOfRangeBehavior,
    SoftSymmetricUnitValue, SourceCharacter, Target, UnitValue,
};
use helgoboss_midi::{Channel, U14, U7};
use reaper_high::Reaper;
use reaper_low::raw;
use reaper_medium::{InitialAction, PromptForActionResult, SectionId};
use rx_util::UnitEvent;
use rxrust::prelude::*;
use std::cell::{Cell, RefCell};
use std::convert::TryInto;

use std::iter;

use std::ptr::null;
use std::rc::Rc;

use crate::application::{
    convert_factor_to_unit_value, convert_unit_value_to_factor, get_fx_label, get_fx_param_label,
    get_guid_based_fx_at_index, get_optional_fx_label, ActivationType, FxAnchorType, MappingModel,
    MidiSourceType, ModeModel, ModifierConditionModel, ProgramConditionModel, ReaperTargetType,
    Session, SharedMapping, SharedSession, SourceCategory, SourceModel, TargetCategory,
    TargetModel, TargetModelWithContext, TrackAnchorType, VirtualControlElementType, WeakSession,
};
use crate::core::Global;
use crate::domain::{
    ActionInvocationType, CompoundMappingTarget, FxAnchor, MappingCompartment, MappingId,
    ProcessorContext, RealearnTarget, ReaperTarget, TargetCharacter, TrackAnchor, TransportAction,
    VirtualControlElement, VirtualFx, VirtualTrack, PLUGIN_PARAMETER_COUNT,
};
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use swell_ui::{SharedView, View, ViewContext, WeakView, Window};

#[derive(Debug)]
pub struct SharedGroupMappingPanel {
    view: ViewContext,
    session: WeakSession,
    item: RefCell<Option<Box<dyn Item>>>,
    is_invoked_programmatically: Cell<bool>,
}

pub trait Item: Debug {
    fn supports_activation(&self) -> bool;
    fn name(&self) -> &str;
    fn set_name(&mut self, name: String);
    fn control_is_enabled(&self) -> bool;
    fn set_control_is_enabled(&mut self, value: bool);
    fn feedback_is_enabled(&self) -> bool;
    fn set_feedback_is_enabled(&mut self, value: bool);
    fn activation_type(&self) -> ActivationType;
    fn set_activation_type(&self, value: ActivationType);
    fn modifier_condition_1(&self) -> ModifierConditionModel;
    fn set_modifier_condition_1(&self, value: ModifierConditionModel);
    fn modifier_condition_2(&self) -> ModifierConditionModel;
    fn set_modifier_condition_2(&self, value: ModifierConditionModel);
    fn program_condition(&self) -> ProgramConditionModel;
    fn set_program_condition(&self, value: ProgramConditionModel);
    fn eel_condition(&self) -> &str;
    fn set_eel_condition(&self, value: String);
}

pub enum ItemProp {
    Name,
    ControlEnabled,
    FeedbackEnabled,
    ActivationType,
    ModifierCondition1,
    ModifierCondition2,
    ProgramCondition,
    EelCondition,
}

impl SharedGroupMappingPanel {
    pub fn new(session: WeakSession) -> SharedGroupMappingPanel {
        SharedGroupMappingPanel {
            view: Default::default(),
            session,
            item: None.into(),
            is_invoked_programmatically: false.into(),
        }
    }

    pub fn is_free(&self) -> bool {
        self.item.borrow().is_none()
    }

    pub fn hide(&self) {
        self.item.replace(None);
    }

    pub fn show(self: SharedView<Self>, item: Box<dyn Item>) {
        self.invoke_programmatically(|| {
            self.invalidate_controls(&*item);
            self.item.replace(Some(item));
            // If this is the first time the window is opened, the following is unnecessary, but if
            // we reuse a window it's important to reset focus for better keyboard control.
            self.view
                .require_control(root::ID_MAPPING_NAME_EDIT_CONTROL)
                .focus();
        });
    }

    /// If you know a function in this view can be invoked by something else than the dialog
    /// process, wrap your function body with this. Basically all pub functions!
    ///
    /// This prevents edit control text change events fired by windows to be processed.
    fn invoke_programmatically(&self, f: impl FnOnce()) {
        self.is_invoked_programmatically.set(true);
        scopeguard::defer! { self.is_invoked_programmatically.set(false); }
        f();
    }

    fn invalidate_controls(&self, item: &dyn Item) {
        self.invalidate_name_edit_control(item);
        self.invalidate_control_enabled_check_box(item);
        self.invalidate_feedback_enabled_check_box(item);
        self.invalidate_activation_controls(item);
    }

    fn init_controls(&self) {
        self.view
            .require_control(root::ID_MAPPING_CONTROL_ENABLED_CHECK_BOX)
            .set_text(format!("{} Control enabled", symbols::ARROW_RIGHT_SYMBOL));
        self.view
            .require_control(root::ID_MAPPING_FEEDBACK_ENABLED_CHECK_BOX)
            .set_text(format!("{} Feedback enabled", symbols::ARROW_LEFT_SYMBOL));
        self.view
            .require_control(root::ID_MAPPING_ACTIVATION_TYPE_COMBO_BOX)
            .fill_combo_box(ActivationType::into_enum_iter());
    }

    fn invalidate_name_edit_control(&self, item: &dyn Item) {
        let c = self
            .view
            .require_control(root::ID_MAPPING_NAME_EDIT_CONTROL);
        c.set_text_if_not_focused(item.name());
    }

    fn invalidate_control_enabled_check_box(&self, item: &dyn Item) {
        self.view
            .require_control(root::ID_MAPPING_CONTROL_ENABLED_CHECK_BOX)
            .set_checked(item.control_is_enabled());
    }

    fn invalidate_feedback_enabled_check_box(&self, item: &dyn Item) {
        self.view
            .require_control(root::ID_MAPPING_FEEDBACK_ENABLED_CHECK_BOX)
            .set_checked(item.feedback_is_enabled());
    }

    fn invalidate_activation_controls(&self, item: &dyn Item) {
        self.invalidate_activation_control_appearance(item);
        self.invalidate_activation_type_combo_box(item);
        self.invalidate_activation_setting_1_controls(item);
        self.invalidate_activation_setting_2_controls(item);
        self.invalidate_activation_eel_condition_edit_control(item);
    }

    fn invalidate_activation_control_appearance(&self, item: &dyn Item) {
        self.invalidate_activation_control_labels(item);
        self.fill_activation_combo_boxes(item);
        self.invalidate_activation_control_visibilities(item);
    }

    fn invalidate_activation_control_labels(&self, item: &dyn Item) {
        use ActivationType::*;
        let label = match item.activation_type() {
            Always => None,
            Modifiers => Some(("Modifier A", "Modifier B")),
            Program => Some(("Bank", "Program")),
            Eel => None,
        };
        if let Some((first, second)) = label {
            self.view
                .require_control(root::ID_MAPPING_ACTIVATION_SETTING_1_LABEL_TEXT)
                .set_text(first);
            self.view
                .require_control(root::ID_MAPPING_ACTIVATION_SETTING_2_LABEL_TEXT)
                .set_text(second);
        }
    }

    fn fill_activation_combo_boxes(&self, item: &dyn Item) {
        use ActivationType::*;
        match item.activation_type() {
            Modifiers => {
                self.fill_combo_box_with_realearn_params(
                    root::ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX,
                    true,
                );
                self.fill_combo_box_with_realearn_params(
                    root::ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX,
                    true,
                );
            }
            Program => {
                self.fill_combo_box_with_realearn_params(
                    root::ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX,
                    false,
                );
                self.view
                    .require_control(root::ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX)
                    .fill_combo_box_with_data_vec(
                        (0..=99).map(|i| (i as isize, i.to_string())).collect(),
                    )
            }
            _ => {}
        };
    }

    fn invalidate_activation_control_visibilities(&self, item: &dyn Item) {
        let show = item.supports_activation();
        let activation_type = item.activation_type();
        self.show_if(
            show,
            &[
                root::ID_MAPPING_ACTIVATION_LABEL,
                root::ID_MAPPING_ACTIVATION_TYPE_COMBO_BOX,
            ],
        );
        self.show_if(
            show && (activation_type == ActivationType::Modifiers
                || activation_type == ActivationType::Program),
            &[
                root::ID_MAPPING_ACTIVATION_SETTING_1_LABEL_TEXT,
                root::ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX,
                root::ID_MAPPING_ACTIVATION_SETTING_2_LABEL_TEXT,
                root::ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX,
            ],
        );
        self.show_if(
            show && activation_type == ActivationType::Modifiers,
            &[
                root::ID_MAPPING_ACTIVATION_SETTING_1_CHECK_BOX,
                root::ID_MAPPING_ACTIVATION_SETTING_2_CHECK_BOX,
            ],
        );
        self.show_if(
            show && activation_type == ActivationType::Eel,
            &[
                root::ID_MAPPING_ACTIVATION_EEL_LABEL_TEXT,
                root::ID_MAPPING_ACTIVATION_EDIT_CONTROL,
            ],
        );
    }

    fn invalidate_activation_type_combo_box(&self, item: &dyn Item) {
        self.view
            .require_control(root::ID_MAPPING_ACTIVATION_TYPE_COMBO_BOX)
            .select_combo_box_item(item.activation_type().into());
    }

    fn invalidate_activation_setting_1_controls(&self, item: &dyn Item) {
        use ActivationType::*;
        match item.activation_type() {
            Modifiers => {
                self.invalidate_mapping_activation_modifier_controls(
                    root::ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX,
                    root::ID_MAPPING_ACTIVATION_SETTING_1_CHECK_BOX,
                    item.modifier_condition_1(),
                );
            }
            Program => {
                let param_index = item.program_condition().param_index();
                self.view
                    .require_control(root::ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX)
                    .select_combo_box_item(param_index as _);
            }
            _ => {}
        };
    }

    fn invalidate_activation_setting_2_controls(&self, item: &dyn Item) {
        use ActivationType::*;
        match item.activation_type() {
            Modifiers => {
                self.invalidate_mapping_activation_modifier_controls(
                    root::ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX,
                    root::ID_MAPPING_ACTIVATION_SETTING_2_CHECK_BOX,
                    item.modifier_condition_2(),
                );
            }
            Program => {
                let program_index = item.program_condition().program_index();
                self.view
                    .require_control(root::ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX)
                    .select_combo_box_item(program_index as _);
            }
            _ => {}
        };
    }

    fn invalidate_mapping_activation_modifier_controls(
        &self,
        combo_box_id: u32,
        check_box_id: u32,
        modifier_condition: ModifierConditionModel,
    ) {
        let b = self.view.require_control(combo_box_id);
        match modifier_condition.param_index() {
            None => {
                b.select_combo_box_item_by_data(-1).unwrap();
            }
            Some(i) => {
                b.select_combo_box_item_by_data(i as _).unwrap();
            }
        };
        self.view
            .require_control(check_box_id)
            .set_checked(modifier_condition.is_on());
    }

    fn is_invoked_programmatically(&self) -> bool {
        self.is_invoked_programmatically.get()
    }

    fn update_control_enabled(&self, item: &mut dyn Item) {
        item.set_control_is_enabled(
            self.view
                .require_control(root::ID_MAPPING_CONTROL_ENABLED_CHECK_BOX)
                .is_checked(),
        );
    }

    fn update_feedback_enabled(&self, item: &mut dyn Item) {
        item.set_feedback_is_enabled(
            self.view
                .require_control(root::ID_MAPPING_FEEDBACK_ENABLED_CHECK_BOX)
                .is_checked(),
        );
    }

    fn update_activation_setting_1_on(&self, item: &mut dyn Item) {
        let checked = self
            .view
            .require_control(root::ID_MAPPING_ACTIVATION_SETTING_1_CHECK_BOX)
            .is_checked();
        item.set_modifier_condition_1(item.modifier_condition_1().with_is_on(checked));
    }

    fn update_activation_setting_2_on(&self, item: &mut dyn Item) {
        let checked = self
            .view
            .require_control(root::ID_MAPPING_ACTIVATION_SETTING_2_CHECK_BOX)
            .is_checked();
        item.set_modifier_condition_2(item.modifier_condition_2().with_is_on(checked));
    }

    fn update_name(&self, item: &mut dyn Item) {
        let value = self
            .view
            .require_control(root::ID_MAPPING_NAME_EDIT_CONTROL)
            .text()
            .unwrap_or_else(|_| "".to_string());
        item.set_name(value);
    }

    fn update_activation_eel_condition(&self, item: &mut dyn Item) {
        let value = self
            .view
            .require_control(root::ID_MAPPING_ACTIVATION_EDIT_CONTROL)
            .text()
            .unwrap_or_else(|_| "".to_string());
        item.set_eel_condition(value);
    }

    fn update_activation_type(&self, item: &mut dyn Item) {
        let b = self
            .view
            .require_control(root::ID_MAPPING_ACTIVATION_TYPE_COMBO_BOX);
        item.set_activation_type(
            b.selected_combo_box_item_index()
                .try_into()
                .expect("invalid activation type"),
        );
    }

    fn update_activation_setting_1_option(&self, item: &mut dyn Item) {
        use ActivationType::*;
        match item.activation_type() {
            Modifiers => {
                self.update_activation_setting_option(
                    root::ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX,
                    || item.modifier_condition_1(),
                    |c| item.set_modifier_condition_1(c),
                );
            }
            Program => {
                let b = self
                    .view
                    .require_control(root::ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX);
                let value = b.selected_combo_box_item_index() as u32;
                item.set_program_condition(item.program_condition().with_param_index(value));
            }
            _ => {}
        };
    }

    fn update_activation_setting_2_option(&self, item: &mut dyn Item) {
        use ActivationType::*;
        match item.activation_type() {
            Modifiers => {
                self.update_activation_setting_option(
                    root::ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX,
                    || item.modifier_condition_2(),
                    |c| item.set_modifier_condition_2(c),
                );
            }
            Program => {
                let b = self
                    .view
                    .require_control(root::ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX);
                let value = b.selected_combo_box_item_index() as u32;
                item.set_program_condition(item.program_condition().with_program_index(value));
            }
            _ => {}
        };
    }

    fn update_activation_setting_option(
        &self,
        combo_box_id: u32,
        get: impl FnOnce() -> ModifierConditionModel,
        set: impl FnOnce(ModifierConditionModel),
    ) {
        let b = self.view.require_control(combo_box_id);
        let value = match b.selected_combo_box_item_data() {
            -1 => None,
            id => Some(id as u32),
        };
        set(get().with_param_index(value));
    }

    fn invalidate_activation_eel_condition_edit_control(&self, item: &dyn Item) {
        self.view
            .require_control(root::ID_MAPPING_ACTIVATION_EDIT_CONTROL)
            .set_text_if_not_focused(item.eel_condition());
    }

    fn show_if(&self, condition: bool, control_resource_ids: &[u32]) {
        for id in control_resource_ids {
            self.view.require_control(*id).set_visible(condition);
        }
    }

    fn if_item_set(&self, f: impl FnOnce(&Self, &dyn Item)) {
        if let Some(item) = self.item.borrow().as_ref() {
            f(self, &(**item));
        }
    }

    fn with_mutable_item(&self, f: impl FnOnce(&Self, &mut dyn Item)) {
        let mut item = self.item.borrow_mut();
        let item = item.as_mut().expect("item not set");
        f(self, &mut (**item));
    }

    pub fn invalidate_due_to_changed_prop(&self, prop: ItemProp) {
        self.if_item_set(|_, item| {
            self.invoke_programmatically(|| {
                use ItemProp::*;
                match prop {
                    Name => self.invalidate_name_edit_control(item),
                    ControlEnabled => self.invalidate_control_enabled_check_box(item),
                    FeedbackEnabled => self.invalidate_feedback_enabled_check_box(item),
                    ActivationType => self.invalidate_activation_controls(item),
                    ModifierCondition1 => self.invalidate_activation_setting_1_controls(item),
                    ModifierCondition2 => self.invalidate_activation_setting_2_controls(item),
                    ProgramCondition => {
                        self.invalidate_activation_setting_1_controls(item);
                        self.invalidate_activation_setting_2_controls(item);
                    }
                    EelCondition => self.invalidate_activation_eel_condition_edit_control(item),
                };
            });
        });
    }

    fn fill_combo_box_with_realearn_params(&self, control_id: u32, with_none: bool) {
        let b = self.view.require_control(control_id);
        let start = if with_none {
            vec![(-1isize, "<None>".to_string())]
        } else {
            vec![]
        };
        let session = self.session();
        let session = session.borrow();
        b.fill_combo_box_with_data_small(start.into_iter().chain((0..PLUGIN_PARAMETER_COUNT).map(
            |i| {
                (
                    i as isize,
                    format!("{}. {}", i + 1, session.get_parameter_name(i)),
                )
            },
        )));
    }

    fn session(&self) -> SharedSession {
        self.session.upgrade().expect("session gone")
    }
}

impl View for SharedGroupMappingPanel {
    fn dialog_resource_id(&self) -> u32 {
        root::ID_SHARED_GROUP_MAPPING_PANEL
    }

    fn view_context(&self) -> &ViewContext {
        &self.view
    }

    fn opened(self: SharedView<Self>, _window: Window) -> bool {
        self.init_controls();
        true
    }

    fn button_clicked(self: SharedView<Self>, resource_id: u32) {
        use root::*;
        match resource_id {
            ID_MAPPING_CONTROL_ENABLED_CHECK_BOX => {
                self.with_mutable_item(Self::update_control_enabled);
            }
            ID_MAPPING_FEEDBACK_ENABLED_CHECK_BOX => {
                self.with_mutable_item(Self::update_feedback_enabled);
            }
            ID_MAPPING_ACTIVATION_SETTING_1_CHECK_BOX => {
                self.with_mutable_item(Self::update_activation_setting_1_on);
            }
            ID_MAPPING_ACTIVATION_SETTING_2_CHECK_BOX => {
                self.with_mutable_item(Self::update_activation_setting_2_on);
            }
            _ => unreachable!(),
        }
    }

    fn option_selected(self: SharedView<Self>, resource_id: u32) {
        use root::*;
        match resource_id {
            ID_MAPPING_ACTIVATION_TYPE_COMBO_BOX => {
                self.with_mutable_item(Self::update_activation_type);
            }
            ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX => {
                self.with_mutable_item(Self::update_activation_setting_1_option);
            }
            ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX => {
                self.with_mutable_item(Self::update_activation_setting_2_option);
            }
            _ => unreachable!(),
        }
    }

    fn edit_control_changed(self: SharedView<Self>, resource_id: u32) -> bool {
        if self.is_invoked_programmatically() {
            // We don't want to continue if the edit control change was not caused by the user.
            // Although the edit control text is changed programmatically, it also triggers the
            // change handler. Ignore it! Most of those events are filtered out already
            // by the dialog proc reentrancy check, but this one is not because the
            // dialog proc is not reentered - we are just reacting (async) to a change.
            return false;
        }
        use root::*;
        match resource_id {
            ID_MAPPING_NAME_EDIT_CONTROL => {
                self.with_mutable_item(Self::update_name);
            }
            ID_MAPPING_ACTIVATION_EDIT_CONTROL => {
                self.with_mutable_item(Self::update_activation_eel_condition);
            }
            _ => return false,
        };
        true
    }

    fn edit_control_focus_killed(self: SharedView<Self>, _resource_id: u32) -> bool {
        // This is also called when the window is hidden.
        // The edit control which is currently edited by the user doesn't get invalidated during
        // `edit_control_changed()`, for good reasons. But as soon as the edit control loses
        // focus, we should invalidate it. This is especially important if the user
        // entered an invalid value. Because we are lazy and edit controls are not
        // manipulated very frequently, we just invalidate all controls.
        // If this fails (because the mapping is not filled anymore), it's not a problem.
        self.if_item_set(Self::invalidate_controls);
        false
    }
}
