mod backend;
mod components;
mod config;
mod theme;
mod views;

use gpui::*;
use views::WorkspaceView;

fn main() {
    env_logger::init();
    log::info!("Démarrage de Shelly GPUI...");

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
