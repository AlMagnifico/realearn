pub mod root {
    #[cfg(target_os = "linux")]
    pub const GLOBAL_X_SCALE: f64 = 1.7500;
    #[cfg(target_os = "linux")]
    pub const GLOBAL_Y_SCALE: f64 = 1.6500;
    #[cfg(target_os = "linux")]
    pub const GLOBAL_WIDTH_SCALE: f64 = 1.7500;
    #[cfg(target_os = "linux")]
    pub const GLOBAL_HEIGHT_SCALE: f64 = 1.6500;
    #[cfg(target_os = "windows")]
    pub const GLOBAL_X_SCALE: f64 = 1.0000;
    #[cfg(target_os = "windows")]
    pub const GLOBAL_Y_SCALE: f64 = 1.0000;
    #[cfg(target_os = "windows")]
    pub const GLOBAL_WIDTH_SCALE: f64 = 1.0000;
    #[cfg(target_os = "windows")]
    pub const GLOBAL_HEIGHT_SCALE: f64 = 1.0000;
    #[cfg(target_os = "macos")]
    pub const GLOBAL_X_SCALE: f64 = 1.6000;
    #[cfg(target_os = "macos")]
    pub const GLOBAL_Y_SCALE: f64 = 1.5200;
    #[cfg(target_os = "macos")]
    pub const GLOBAL_WIDTH_SCALE: f64 = 1.6000;
    #[cfg(target_os = "macos")]
    pub const GLOBAL_HEIGHT_SCALE: f64 = 1.5200;
    #[cfg(target_os = "linux")]
    pub const MAPPING_PANEL_X_SCALE: f64 = 1.7500;
    #[cfg(target_os = "linux")]
    pub const MAPPING_PANEL_Y_SCALE: f64 = 1.6500;
    #[cfg(target_os = "linux")]
    pub const MAPPING_PANEL_WIDTH_SCALE: f64 = 1.7500;
    #[cfg(target_os = "linux")]
    pub const MAPPING_PANEL_HEIGHT_SCALE: f64 = 1.6500;
    #[cfg(target_os = "windows")]
    pub const MAPPING_PANEL_X_SCALE: f64 = 1.0000;
    #[cfg(target_os = "windows")]
    pub const MAPPING_PANEL_Y_SCALE: f64 = 0.8000;
    #[cfg(target_os = "windows")]
    pub const MAPPING_PANEL_WIDTH_SCALE: f64 = 1.0000;
    #[cfg(target_os = "windows")]
    pub const MAPPING_PANEL_HEIGHT_SCALE: f64 = 0.8000;
    #[cfg(target_os = "macos")]
    pub const MAPPING_PANEL_X_SCALE: f64 = 1.6000;
    #[cfg(target_os = "macos")]
    pub const MAPPING_PANEL_Y_SCALE: f64 = 1.4000;
    #[cfg(target_os = "macos")]
    pub const MAPPING_PANEL_WIDTH_SCALE: f64 = 1.6000;
    #[cfg(target_os = "macos")]
    pub const MAPPING_PANEL_HEIGHT_SCALE: f64 = 1.4000;
    pub const ID_GROUP_PANEL: u32 = 30000;
    pub const ID_GROUP_PANEL_OK: u32 = 30001;
    pub const ID_HEADER_PANEL: u32 = 30040;
    pub const ID_CONTROL_DEVICE_COMBO_BOX: u32 = 30003;
    pub const ID_FEEDBACK_DEVICE_COMBO_BOX: u32 = 30005;
    pub const ID_IMPORT_BUTTON: u32 = 30006;
    pub const ID_EXPORT_BUTTON: u32 = 30007;
    pub const ID_PROJECTION_BUTTON: u32 = 30008;
    pub const ID_LET_THROUGH_LABEL_TEXT: u32 = 30009;
    pub const ID_LET_MATCHED_EVENTS_THROUGH_CHECK_BOX: u32 = 30010;
    pub const ID_LET_UNMATCHED_EVENTS_THROUGH_CHECK_BOX: u32 = 30011;
    pub const ID_CONTROLLER_COMPARTMENT_RADIO_BUTTON: u32 = 30013;
    pub const ID_MAIN_COMPARTMENT_RADIO_BUTTON: u32 = 30014;
    pub const ID_PRESET_LABEL_TEXT: u32 = 30015;
    pub const ID_PRESET_COMBO_BOX: u32 = 30016;
    pub const ID_PRESET_SAVE_AS_BUTTON: u32 = 30017;
    pub const ID_PRESET_SAVE_BUTTON: u32 = 30018;
    pub const ID_PRESET_DELETE_BUTTON: u32 = 30019;
    pub const ID_AUTO_LOAD_LABEL_TEXT: u32 = 30020;
    pub const ID_AUTO_LOAD_COMBO_BOX: u32 = 30021;
    pub const ID_GROUP_COMBO_BOX: u32 = 30023;
    pub const ID_GROUP_ADD_BUTTON: u32 = 30024;
    pub const ID_GROUP_DELETE_BUTTON: u32 = 30025;
    pub const ID_GROUP_EDIT_BUTTON: u32 = 30026;
    pub const ID_ADD_MAPPING_BUTTON: u32 = 30028;
    pub const ID_LEARN_MANY_MAPPINGS_BUTTON: u32 = 30029;
    pub const ID_HEADER_SEARCH_EDIT_CONTROL: u32 = 30031;
    pub const ID_CLEAR_SEARCH_BUTTON: u32 = 30032;
    pub const ID_FILTER_BY_SOURCE_BUTTON: u32 = 30033;
    pub const ID_CLEAR_SOURCE_FILTER_BUTTON: u32 = 30034;
    pub const ID_FILTER_BY_TARGET_BUTTON: u32 = 30035;
    pub const ID_CLEAR_TARGET_FILTER_BUTTON: u32 = 30036;
    pub const ID_MAPPING_PANEL: u32 = 30194;
    pub const ID_MAPPING_FEEDBACK_SEND_BEHAVIOR_COMBO_BOX: u32 = 30043;
    pub const ID_MAPPING_SHOW_IN_PROJECTION_CHECK_BOX: u32 = 30044;
    pub const ID_MAPPING_ADVANCED_BUTTON: u32 = 30045;
    pub const ID_MAPPING_FIND_IN_LIST_BUTTON: u32 = 30046;
    pub const ID_SOURCE_LEARN_BUTTON: u32 = 30048;
    pub const ID_SOURCE_CATEGORY_COMBO_BOX: u32 = 30050;
    pub const ID_SOURCE_TYPE_LABEL_TEXT: u32 = 30051;
    pub const ID_SOURCE_TYPE_COMBO_BOX: u32 = 30052;
    pub const ID_SOURCE_MIDI_MESSAGE_TYPE_LABEL_TEXT: u32 = 30053;
    pub const ID_SOURCE_CHANNEL_LABEL: u32 = 30054;
    pub const ID_SOURCE_CHANNEL_COMBO_BOX: u32 = 30055;
    pub const ID_SOURCE_LINE_3_EDIT_CONTROL: u32 = 30056;
    pub const ID_SOURCE_MIDI_CLOCK_TRANSPORT_MESSAGE_TYPE_COMBOX_BOX: u32 = 30057;
    pub const ID_SOURCE_NOTE_OR_CC_NUMBER_LABEL_TEXT: u32 = 30058;
    pub const ID_SOURCE_RPN_CHECK_BOX: u32 = 30059;
    pub const ID_SOURCE_LINE_4_COMBO_BOX_1: u32 = 30060;
    pub const ID_SOURCE_NUMBER_EDIT_CONTROL: u32 = 30061;
    pub const ID_SOURCE_NUMBER_COMBO_BOX: u32 = 30062;
    pub const ID_SOURCE_LINE_4_BUTTON: u32 = 30063;
    pub const ID_SOURCE_CHARACTER_LABEL_TEXT: u32 = 30064;
    pub const ID_SOURCE_CHARACTER_COMBO_BOX: u32 = 30065;
    pub const ID_SOURCE_LINE_5_EDIT_CONTROL: u32 = 30066;
    pub const ID_SOURCE_14_BIT_CHECK_BOX: u32 = 30067;
    pub const ID_SOURCE_OSC_ADDRESS_LABEL_TEXT: u32 = 30068;
    pub const ID_SOURCE_OSC_ADDRESS_PATTERN_EDIT_CONTROL: u32 = 30069;
    pub const ID_SOURCE_SCRIPT_DETAIL_BUTTON: u32 = 30070;
    pub const ID_TARGET_LEARN_BUTTON: u32 = 30072;
    pub const ID_TARGET_OPEN_BUTTON: u32 = 30073;
    pub const ID_TARGET_HINT: u32 = 30074;
    pub const ID_TARGET_CATEGORY_COMBO_BOX: u32 = 30076;
    pub const ID_TARGET_TYPE_COMBO_BOX: u32 = 30077;
    pub const ID_TARGET_LINE_2_LABEL_2: u32 = 30078;
    pub const ID_TARGET_LINE_2_LABEL_3: u32 = 30079;
    pub const ID_TARGET_LINE_2_LABEL_1: u32 = 30080;
    pub const ID_TARGET_LINE_2_COMBO_BOX_1: u32 = 30081;
    pub const ID_TARGET_LINE_2_EDIT_CONTROL: u32 = 30082;
    pub const ID_TARGET_LINE_2_COMBO_BOX_2: u32 = 30083;
    pub const ID_TARGET_LINE_2_BUTTON: u32 = 30084;
    pub const ID_TARGET_LINE_3_LABEL_1: u32 = 30085;
    pub const ID_TARGET_LINE_3_COMBO_BOX_1: u32 = 30086;
    pub const ID_TARGET_LINE_3_EDIT_CONTROL: u32 = 30087;
    pub const ID_TARGET_LINE_3_COMBO_BOX_2: u32 = 30088;
    pub const ID_TARGET_LINE_3_LABEL_2: u32 = 30089;
    pub const ID_TARGET_LINE_3_LABEL_3: u32 = 30090;
    pub const ID_TARGET_LINE_3_BUTTON: u32 = 30091;
    pub const ID_TARGET_LINE_4_LABEL_1: u32 = 30092;
    pub const ID_TARGET_LINE_4_COMBO_BOX_1: u32 = 30093;
    pub const ID_TARGET_LINE_4_EDIT_CONTROL: u32 = 30094;
    pub const ID_TARGET_LINE_4_COMBO_BOX_2: u32 = 30095;
    pub const ID_TARGET_LINE_4_LABEL_2: u32 = 30096;
    pub const ID_TARGET_LINE_4_BUTTON: u32 = 30097;
    pub const ID_TARGET_LINE_4_LABEL_3: u32 = 30098;
    pub const ID_TARGET_LINE_5_LABEL_1: u32 = 30099;
    pub const ID_TARGET_LINE_5_EDIT_CONTROL: u32 = 30100;
    pub const ID_TARGET_CHECK_BOX_1: u32 = 30101;
    pub const ID_TARGET_CHECK_BOX_2: u32 = 30102;
    pub const ID_TARGET_CHECK_BOX_3: u32 = 30103;
    pub const ID_TARGET_CHECK_BOX_4: u32 = 30104;
    pub const ID_TARGET_CHECK_BOX_5: u32 = 30105;
    pub const ID_TARGET_CHECK_BOX_6: u32 = 30106;
    pub const ID_TARGET_VALUE_LABEL_TEXT: u32 = 30107;
    pub const ID_TARGET_VALUE_OFF_BUTTON: u32 = 30108;
    pub const ID_TARGET_VALUE_ON_BUTTON: u32 = 30109;
    pub const ID_TARGET_VALUE_SLIDER_CONTROL: u32 = 30110;
    pub const ID_TARGET_VALUE_EDIT_CONTROL: u32 = 30111;
    pub const ID_TARGET_VALUE_TEXT: u32 = 30112;
    pub const ID_TARGET_UNIT_BUTTON: u32 = 30113;
    pub const ID_SETTINGS_RESET_BUTTON: u32 = 30115;
    pub const ID_SETTINGS_SOURCE_LABEL: u32 = 30116;
    #[allow(dead_code)]
    pub const ID_SETTINGS_SOURCE_GROUP: u32 = 30117;
    pub const ID_SETTINGS_SOURCE_MIN_LABEL: u32 = 30118;
    pub const ID_SETTINGS_MIN_SOURCE_VALUE_SLIDER_CONTROL: u32 = 30119;
    pub const ID_SETTINGS_MIN_SOURCE_VALUE_EDIT_CONTROL: u32 = 30120;
    pub const ID_SETTINGS_SOURCE_MAX_LABEL: u32 = 30121;
    pub const ID_SETTINGS_MAX_SOURCE_VALUE_SLIDER_CONTROL: u32 = 30122;
    pub const ID_SETTINGS_MAX_SOURCE_VALUE_EDIT_CONTROL: u32 = 30123;
    pub const ID_MODE_OUT_OF_RANGE_LABEL_TEXT: u32 = 30124;
    pub const ID_MODE_OUT_OF_RANGE_COMBOX_BOX: u32 = 30125;
    pub const ID_MODE_GROUP_INTERACTION_LABEL_TEXT: u32 = 30126;
    pub const ID_MODE_GROUP_INTERACTION_COMBO_BOX: u32 = 30127;
    pub const ID_SETTINGS_TARGET_LABEL_TEXT: u32 = 30128;
    pub const ID_SETTINGS_TARGET_SEQUENCE_LABEL_TEXT: u32 = 30129;
    pub const ID_MODE_TARGET_SEQUENCE_EDIT_CONTROL: u32 = 30130;
    #[allow(dead_code)]
    pub const ID_SETTINGS_TARGET_GROUP: u32 = 30131;
    pub const ID_SETTINGS_MIN_TARGET_LABEL_TEXT: u32 = 30132;
    pub const ID_SETTINGS_MIN_TARGET_VALUE_SLIDER_CONTROL: u32 = 30133;
    pub const ID_SETTINGS_MIN_TARGET_VALUE_EDIT_CONTROL: u32 = 30134;
    pub const ID_SETTINGS_MIN_TARGET_VALUE_TEXT: u32 = 30135;
    pub const ID_SETTINGS_MAX_TARGET_LABEL_TEXT: u32 = 30136;
    pub const ID_SETTINGS_MAX_TARGET_VALUE_SLIDER_CONTROL: u32 = 30137;
    pub const ID_SETTINGS_MAX_TARGET_VALUE_EDIT_CONTROL: u32 = 30138;
    pub const ID_SETTINGS_MAX_TARGET_VALUE_TEXT: u32 = 30139;
    pub const ID_SETTINGS_REVERSE_CHECK_BOX: u32 = 30140;
    pub const IDC_MODE_FEEDBACK_TYPE_COMBO_BOX: u32 = 30141;
    pub const ID_MODE_EEL_FEEDBACK_TRANSFORMATION_EDIT_CONTROL: u32 = 30142;
    pub const IDC_MODE_FEEDBACK_TYPE_BUTTON: u32 = 30143;
    pub const ID_MODE_KNOB_FADER_GROUP_BOX: u32 = 30144;
    pub const ID_SETTINGS_MODE_LABEL: u32 = 30145;
    pub const ID_SETTINGS_MODE_COMBO_BOX: u32 = 30146;
    pub const ID_SETTINGS_TARGET_JUMP_LABEL_TEXT: u32 = 30147;
    #[allow(dead_code)]
    pub const ID_SETTINGS_TARGET_JUMP_GROUP: u32 = 30148;
    pub const ID_SETTINGS_MIN_TARGET_JUMP_LABEL_TEXT: u32 = 30149;
    pub const ID_SETTINGS_MIN_TARGET_JUMP_SLIDER_CONTROL: u32 = 30150;
    pub const ID_SETTINGS_MIN_TARGET_JUMP_EDIT_CONTROL: u32 = 30151;
    pub const ID_SETTINGS_MIN_TARGET_JUMP_VALUE_TEXT: u32 = 30152;
    pub const ID_SETTINGS_MAX_TARGET_JUMP_LABEL_TEXT: u32 = 30153;
    pub const ID_SETTINGS_MAX_TARGET_JUMP_SLIDER_CONTROL: u32 = 30154;
    pub const ID_SETTINGS_MAX_TARGET_JUMP_EDIT_CONTROL: u32 = 30155;
    pub const ID_SETTINGS_MAX_TARGET_JUMP_VALUE_TEXT: u32 = 30156;
    pub const ID_MODE_TAKEOVER_LABEL: u32 = 30157;
    pub const ID_MODE_TAKEOVER_MODE: u32 = 30158;
    pub const ID_SETTINGS_ROUND_TARGET_VALUE_CHECK_BOX: u32 = 30159;
    pub const ID_MODE_EEL_CONTROL_TRANSFORMATION_LABEL: u32 = 30160;
    pub const ID_MODE_EEL_CONTROL_TRANSFORMATION_EDIT_CONTROL: u32 = 30161;
    pub const ID_MODE_RELATIVE_GROUP_BOX: u32 = 30162;
    pub const ID_SETTINGS_STEP_SIZE_LABEL_TEXT: u32 = 30163;
    #[allow(dead_code)]
    pub const ID_SETTINGS_STEP_SIZE_GROUP: u32 = 30164;
    pub const ID_SETTINGS_MIN_STEP_SIZE_LABEL_TEXT: u32 = 30165;
    pub const ID_SETTINGS_MIN_STEP_SIZE_SLIDER_CONTROL: u32 = 30166;
    pub const ID_SETTINGS_MIN_STEP_SIZE_EDIT_CONTROL: u32 = 30167;
    pub const ID_SETTINGS_MIN_STEP_SIZE_VALUE_TEXT: u32 = 30168;
    pub const ID_SETTINGS_MAX_STEP_SIZE_LABEL_TEXT: u32 = 30169;
    pub const ID_SETTINGS_MAX_STEP_SIZE_SLIDER_CONTROL: u32 = 30170;
    pub const ID_SETTINGS_MAX_STEP_SIZE_EDIT_CONTROL: u32 = 30171;
    pub const ID_SETTINGS_MAX_STEP_SIZE_VALUE_TEXT: u32 = 30172;
    pub const ID_MODE_RELATIVE_FILTER_COMBO_BOX: u32 = 30173;
    pub const ID_SETTINGS_ROTATE_CHECK_BOX: u32 = 30174;
    pub const ID_SETTINGS_MAKE_ABSOLUTE_CHECK_BOX: u32 = 30175;
    pub const ID_MODE_BUTTON_GROUP_BOX: u32 = 30176;
    pub const ID_MODE_FIRE_COMBO_BOX: u32 = 30177;
    pub const ID_MODE_BUTTON_FILTER_COMBO_BOX: u32 = 30178;
    pub const ID_MODE_FIRE_LINE_2_LABEL_1: u32 = 30179;
    pub const ID_MODE_FIRE_LINE_2_SLIDER_CONTROL: u32 = 30180;
    pub const ID_MODE_FIRE_LINE_2_EDIT_CONTROL: u32 = 30181;
    pub const ID_MODE_FIRE_LINE_2_LABEL_2: u32 = 30182;
    pub const ID_MODE_FIRE_LINE_3_LABEL_1: u32 = 30183;
    pub const ID_MODE_FIRE_LINE_3_SLIDER_CONTROL: u32 = 30184;
    pub const ID_MODE_FIRE_LINE_3_EDIT_CONTROL: u32 = 30185;
    pub const ID_MODE_FIRE_LINE_3_LABEL_2: u32 = 30186;
    pub const ID_MAPPING_HELP_SUBJECT_LABEL: u32 = 30187;
    pub const IDC_MAPPING_MATCHED_INDICATOR_TEXT: u32 = 30188;
    pub const ID_MAPPING_HELP_APPLICABLE_TO_LABEL: u32 = 30189;
    pub const ID_MAPPING_HELP_APPLICABLE_TO_COMBO_BOX: u32 = 30190;
    pub const ID_MAPPING_HELP_CONTENT_LABEL: u32 = 30191;
    pub const ID_MAPPING_PANEL_OK: u32 = 30192;
    pub const IDC_MAPPING_ENABLED_CHECK_BOX: u32 = 30193;
    pub const ID_MAPPING_ROW_PANEL: u32 = 30212;
    pub const ID_MAPPING_ROW_MAPPING_LABEL: u32 = 30195;
    pub const IDC_MAPPING_ROW_ENABLED_CHECK_BOX: u32 = 30196;
    pub const ID_MAPPING_ROW_EDIT_BUTTON: u32 = 30197;
    pub const ID_MAPPING_ROW_DUPLICATE_BUTTON: u32 = 30198;
    pub const ID_MAPPING_ROW_REMOVE_BUTTON: u32 = 30199;
    pub const ID_MAPPING_ROW_LEARN_SOURCE_BUTTON: u32 = 30200;
    pub const ID_MAPPING_ROW_LEARN_TARGET_BUTTON: u32 = 30201;
    pub const ID_MAPPING_ROW_CONTROL_CHECK_BOX: u32 = 30202;
    pub const ID_MAPPING_ROW_FEEDBACK_CHECK_BOX: u32 = 30203;
    pub const ID_MAPPING_ROW_SOURCE_LABEL_TEXT: u32 = 30204;
    pub const ID_MAPPING_ROW_TARGET_LABEL_TEXT: u32 = 30205;
    pub const ID_MAPPING_ROW_DIVIDER: u32 = 30206;
    pub const ID_MAPPING_ROW_GROUP_LABEL: u32 = 30207;
    pub const IDC_MAPPING_ROW_MATCHED_INDICATOR_TEXT: u32 = 30208;
    pub const ID_UP_BUTTON: u32 = 30210;
    pub const ID_DOWN_BUTTON: u32 = 30211;
    pub const ID_MAPPING_ROWS_PANEL: u32 = 30215;
    pub const ID_DISPLAY_ALL_GROUPS_BUTTON: u32 = 30213;
    pub const ID_GROUP_IS_EMPTY_TEXT: u32 = 30214;
    pub const ID_MESSAGE_PANEL: u32 = 30217;
    pub const ID_MESSAGE_TEXT: u32 = 30216;
    pub const ID_SHARED_GROUP_MAPPING_PANEL: u32 = 30234;
    pub const ID_MAPPING_NAME_EDIT_CONTROL: u32 = 30219;
    pub const ID_MAPPING_TAGS_EDIT_CONTROL: u32 = 30221;
    pub const ID_MAPPING_CONTROL_ENABLED_CHECK_BOX: u32 = 30222;
    pub const ID_MAPPING_FEEDBACK_ENABLED_CHECK_BOX: u32 = 30223;
    pub const ID_MAPPING_ACTIVATION_LABEL: u32 = 30224;
    pub const ID_MAPPING_ACTIVATION_TYPE_COMBO_BOX: u32 = 30225;
    pub const ID_MAPPING_ACTIVATION_SETTING_1_LABEL_TEXT: u32 = 30226;
    pub const ID_MAPPING_ACTIVATION_SETTING_1_COMBO_BOX: u32 = 30227;
    pub const ID_MAPPING_ACTIVATION_SETTING_1_CHECK_BOX: u32 = 30228;
    pub const ID_MAPPING_ACTIVATION_SETTING_2_LABEL_TEXT: u32 = 30229;
    pub const ID_MAPPING_ACTIVATION_SETTING_2_COMBO_BOX: u32 = 30230;
    pub const ID_MAPPING_ACTIVATION_SETTING_2_CHECK_BOX: u32 = 30231;
    pub const ID_MAPPING_ACTIVATION_EEL_LABEL_TEXT: u32 = 30232;
    pub const ID_MAPPING_ACTIVATION_EDIT_CONTROL: u32 = 30233;
    pub const ID_MAIN_PANEL: u32 = 30239;
    pub const ID_MAIN_PANEL_VERSION_TEXT: u32 = 30235;
    pub const ID_MAIN_PANEL_STATUS_TEXT: u32 = 30236;
    pub const IDC_EDIT_TAGS_BUTTON: u32 = 30237;
    pub const ID_YAML_EDITOR_PANEL: u32 = 30244;
    pub const ID_YAML_TEXT_EDITOR_BUTTON: u32 = 30240;
    pub const ID_YAML_EDIT_CONTROL: u32 = 30241;
    pub const ID_YAML_HELP_BUTTON: u32 = 30242;
    pub const ID_YAML_EDIT_INFO_TEXT: u32 = 30243;
}
