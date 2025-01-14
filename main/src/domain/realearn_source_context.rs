use helgoboss_learn::devices::x_touch::XTouchMackieLcdState;
use reaper_medium::MidiOutputDeviceId;
use std::collections::HashMap;

/// Global state about sources.
#[derive(Default)]
pub struct RealearnSourceState {
    x_touch_mackie_lcd_state_by_device: HashMap<MidiOutputDeviceId, XTouchMackieLcdState>,
}

impl RealearnSourceState {
    pub fn get_x_touch_mackie_lcd_state_mut(
        &mut self,
        device: MidiOutputDeviceId,
    ) -> &mut XTouchMackieLcdState {
        self.x_touch_mackie_lcd_state_by_device
            .entry(device)
            .or_default()
    }
}
