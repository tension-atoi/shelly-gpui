#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_must_use)]

mod backend;
mod components;
mod config;
mod models;
mod theme;
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
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| {
            cx.new(|cx| WorkspaceView::new(cx))
        });
    });
}
