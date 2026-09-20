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

use gpui::*;
use views::WorkspaceView;

fn main() {
    env_logger::init();
    log::info!("Starting Shelly GPUI...");

    // Initialise et entre dans le contexte du runtime Tokio pour les tâches et processus async
    let _rt_guard = backend::process::runtime().enter();

    let config = crate::config::ConfigManager::load_gpui_config_sanitized();
    let initial_config = config.clone();

    Application::new()
        .with_assets(crate::icons::AppIcons::new())
        .run(move |cx: &mut App| {
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(config.window_width), px(config.window_height)),
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
                cx.new(|cx| WorkspaceView::with_config(initial_config, cx))
            });
        });
}
