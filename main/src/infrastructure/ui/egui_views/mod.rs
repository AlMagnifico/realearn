use egui::{Context, Visuals};
use reaper_low::firewall;
use swell_ui::{SwellWindow, Window};

pub mod advanced_script_editor;
pub mod target_filter_panel;

pub fn open<S: Send + 'static>(
    window: Window,
    title: impl Into<String>,
    state: S,
    run_ui: impl Fn(&egui::Context, &mut S) + Send + Sync + 'static,
) {
    let title = title.into();
    window.set_text(title.as_str());
    let window_size = window.size();
    #[cfg(windows)]
    let (width, height, scale) = {
        let dpi_factor = window.dpi_scaling_factor();
        let width = window_size.width.get() as f64 / dpi_factor;
        let height = window_size.height.get() as f64 / dpi_factor;
        let scale = baseview::WindowScalePolicy::ScaleFactor(dpi_factor);
        (width, height, scale)
    };
    #[cfg(not(windows))]
    let (width, height, scale) = {
        let width = window_size.width.get() as f64;
        let height = window_size.height.get() as f64;
        let scale = baseview::WindowScalePolicy::SystemScaleFactor;
        (width, height, scale)
    };
    let settings = baseview::WindowOpenOptions {
        title,
        size: baseview::Size::new(width, height),
        scale,
        gl_config: Some(Default::default()),
    };
    let window = get_egui_parent_window(window);
    egui_baseview::EguiWindow::open_parented(
        &window,
        settings,
        state,
        |ctx: &egui::Context, _queue: &mut egui_baseview::Queue, _state: &mut S| {
            firewall(|| {
                init_ui(ctx, Window::dark_mode_is_enabled());
            });
        },
        move |ctx: &egui::Context, _queue: &mut egui_baseview::Queue, state: &mut S| {
            firewall(|| {
                run_ui(ctx, state);
            });
        },
    );
}

fn get_egui_parent_window(window: Window) -> SwellWindow {
    #[cfg(target_os = "linux")]
    {
        let x_bridge_window = swell_ui::XBridgeWindow::create(window).unwrap();
        SwellWindow::XBridge(x_bridge_window)
    }
    #[cfg(not(target_os = "linux"))]
    {
        SwellWindow::Normal(window)
    }
}

/// To be called in the `resized` callback of the container window. Takes care of resizing
/// the egui child window to exactly the same size as parent. Works at least on Windows.
pub fn on_parent_window_resize(parent: Window) -> bool {
    #[cfg(windows)]
    {
        parent.resize_first_child_according_to_parent();
        true
    }
    #[cfg(not(windows))]
    {
        let _ = parent;
        false
    }
}

fn init_ui(ctx: &Context, dark_mode_is_enabled: bool) {
    let mut style: egui::Style = (*ctx.style()).clone();
    style.visuals = if dark_mode_is_enabled {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    ctx.set_style(style);
}
