#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_must_use)]
#![recursion_limit = "2048"]

mod backend;
mod components;
mod config;
pub mod icons;
mod state;
mod theme;
pub mod ui_metrics;
mod views;

use crate::components::search_input::*;
use gpui::*;
use views::WorkspaceView;

fn main() {
    env_logger::init();
    log::info!("Starting Shelly GPUI...");

    // Initialise et entre dans le contexte du runtime Tokio pour les tâches et processus async
    let _rt_guard = backend::process::runtime().enter();

    let shelly_settings = crate::config::ConfigManager::load_shelly_settings();
    let gpui_config = crate::config::ConfigManager::load_gpui_config_sanitized();
    let initial_shelly_settings = shelly_settings.clone();
    let initial_gpui_config = gpui_config.clone();

    Application::new()
        .with_assets(crate::icons::AppIcons::new())
        .run(move |cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("backspace", Backspace, Some("SearchInput")),
                KeyBinding::new("delete", Delete, Some("SearchInput")),
                KeyBinding::new("left", Left, Some("SearchInput")),
                KeyBinding::new("right", Right, Some("SearchInput")),
                KeyBinding::new("shift-left", SelectLeft, Some("SearchInput")),
                KeyBinding::new("shift-right", SelectRight, Some("SearchInput")),
                KeyBinding::new("ctrl-a", SelectAll, Some("SearchInput")),
                KeyBinding::new("cmd-a", SelectAll, Some("SearchInput")),
                KeyBinding::new("home", Home, Some("SearchInput")),
                KeyBinding::new("end", End, Some("SearchInput")),
                KeyBinding::new("ctrl-v", Paste, Some("SearchInput")),
                KeyBinding::new("cmd-v", Paste, Some("SearchInput")),
                KeyBinding::new("ctrl-c", Copy, Some("SearchInput")),
                KeyBinding::new("cmd-c", Copy, Some("SearchInput")),
                KeyBinding::new("ctrl-x", Cut, Some("SearchInput")),
                KeyBinding::new("cmd-x", Cut, Some("SearchInput")),
                KeyBinding::new("escape", Escape, Some("SearchInput")),
                KeyBinding::new("enter", Enter, Some("SearchInput")),
            ]);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(gpui_config.window_width), px(gpui_config.window_height)),
                })),
                window_min_size: Some(size(
                    px(crate::config::MIN_WINDOW_WIDTH),
                    px(crate::config::MIN_WINDOW_HEIGHT),
                )),
                titlebar: Some(TitlebarOptions {
                    title: Some("Shelly — Universal Package Manager".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                app_id: Some("shelly-gpui".into()),
                ..Default::default()
            };

            let _ = cx.open_window(options, move |_, cx| {
                cx.new(|cx| {
                    WorkspaceView::with_config(initial_shelly_settings, initial_gpui_config, cx)
                })
            });
        });
}
