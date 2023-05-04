use crate::base::{blocking_lock_arc, Global};
use crate::domain::enigo::EnigoMouse;
use crate::domain::pot::spawn_in_pot_worker;
use crate::domain::{Mouse, MouseCursorPosition};
use indexmap::{IndexMap, IndexSet};
use realearn_api::persistence::MouseButton;
use reaper_high::{Fx, Reaper};
use std::error::Error;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub type SharedPresetCrawlingState = Arc<Mutex<PresetCrawlingState>>;

#[derive(Debug)]
pub struct PresetCrawlingState {
    crawled_presets: IndexMap<String, CrawledPreset>,
    crawling_finished: bool,
    duplicate_preset_names: IndexSet<String>,
}

impl PresetCrawlingState {
    pub fn new() -> SharedPresetCrawlingState {
        let state = Self {
            crawled_presets: Default::default(),
            crawling_finished: false,
            duplicate_preset_names: Default::default(),
        };
        Arc::new(Mutex::new(state))
    }

    pub fn last_crawled_preset(&self) -> Option<&CrawledPreset> {
        let last = self.crawled_presets.last()?;
        Some(last.1)
    }

    pub fn pop_crawled_preset(&mut self) -> Option<CrawledPreset> {
        let last = self.crawled_presets.pop()?;
        Some(last.1)
    }

    pub fn crawling_is_finished(&self) -> bool {
        self.crawling_finished
    }

    pub fn preset_count(&self) -> u32 {
        self.crawled_presets.len() as _
    }

    pub fn duplicate_name_count(&self) -> u32 {
        self.duplicate_preset_names.len() as _
    }

    pub fn duplicate_names(&self) -> &IndexSet<String> {
        &self.duplicate_preset_names
    }

    /// Returns `false` if crawling should stop.
    fn add_preset(&mut self, preset: CrawledPreset) -> bool {
        // Give stop signal if we reached the end of the list or are at its beginning again.
        if let Some((_, last_preset)) = self.crawled_presets.last() {
            if &preset == last_preset {
                // Same like last crawled preset. Either the "Next preset" button doesn't
                // work at all or we have reached the end of the preset list.
                self.crawling_finished = true;
                return false;
            }
            if self.crawled_presets.len() > 1 {
                let (_, first_preset) = self.crawled_presets.first().expect("must exist");
                if &preset == first_preset {
                    // Same like first crawled preset. We are back at the first preset again,
                    // no need to crawl more.
                    self.crawling_finished = true;
                    return false;
                }
            }
        }
        if self.crawled_presets.contains_key(&preset.name) {
            // Duplicate name. Skip preset!
            self.duplicate_preset_names.insert(preset.name);
        } else {
            // Add preset
            self.crawled_presets.insert(preset.name.clone(), preset);
        }
        true
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct CrawledPreset {
    name: String,
    fx_chunk: String,
}

impl CrawledPreset {
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub fn crawl_presets(
    fx: Fx,
    next_preset_cursor_pos: MouseCursorPosition,
    state: SharedPresetCrawlingState,
    bring_focus_back_to_crawler: impl Fn() + 'static,
) {
    Global::future_support().spawn_in_main_thread_from_main_thread(async move {
        let mut mouse = EnigoMouse::default();
        loop {
            // Get preset name
            let name = fx
                .preset_name()
                .ok_or("couldn't get preset name")?
                .into_string();
            // Query chunk and save it in temporary file
            let fx_chunk = fx.chunk()?;
            // Build crawled preset
            let crawled_preset = CrawledPreset {
                name,
                fx_chunk: fx_chunk.to_string(),
            };
            if !blocking_lock_arc(&state, "crawl_presets").add_preset(crawled_preset) {
                // Finished
                bring_focus_back_to_crawler();
                break;
            }
            // Click "Next preset" button
            fx.show_in_floating_window();
            mouse.set_cursor_position(next_preset_cursor_pos)?;
            moment().await;
            mouse.press(MouseButton::Left)?;
            moment().await;
            mouse.release(MouseButton::Left)?;
            moment().await;
        }
        Ok(())
    });
}

pub fn import_crawled_presets(
    fx: Fx,
    state: SharedPresetCrawlingState,
) -> Result<(), Box<dyn Error>> {
    let fx_chain_dir = Reaper::get().resource_path().join("FXChains");
    let fx_info = fx.info()?;
    spawn_in_pot_worker(async move {
        loop {
            let p = blocking_lock_arc(&state, "import_crawled_presets").pop_crawled_preset();
            let Some(p) = p else {
                break;
            };
            let file_name = format!("{}.RfxChain", p.name);
            let dest_dir_path = fx_chain_dir.join(&fx_info.effect_name);
            fs::create_dir_all(&dest_dir_path)?;
            let dest_file_path = dest_dir_path.join(file_name);
            fs::write(dest_file_path, p.fx_chunk)?;
        }
        Ok(())
    });
    Ok(())
}

async fn moment() {
    millis(200).await;
}

async fn millis(amount: u64) {
    futures_timer::Delay::new(Duration::from_millis(amount)).await;
}
