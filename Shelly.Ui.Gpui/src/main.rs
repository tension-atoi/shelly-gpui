#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_must_use)]
#![recursion_limit = "1024"]

mod backend;
mod components;
mod config;
mod state;
mod theme;
pub mod ui_metrics;
mod views;

use gpui::*;
use views::WorkspaceView;

fn main() {
    env_logger::init();
    log::info!("Démarrage de Shelly GPUI...");

    // Initialise et entre dans le contexte du runtime Tokio pour les tâches et processus async
    let _rt_guard = backend::process::runtime().enter();

    Application::new().run(|cx: &mut App| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::default(),
                size: size(px(1280.0), px(840.0)),
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("Shelly — Gestionnaire de paquets Arch Linux (GPUI)".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            app_id: Some("shelly-gpui".into()),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| cx.new(WorkspaceView::new));
    });
}
