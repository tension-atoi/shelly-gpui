use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

pub struct DiagnosticsHudProps<'a> {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub active_rows: usize,
    pub memory_mb: f32,
    pub theme: &'a Theme,
    pub on_close: Option<Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>>,
}

pub struct DiagnosticsHud;

impl DiagnosticsHud {
    pub fn read_rss_memory_mb() -> f32 {
        if let Ok(content) = std::fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(pages) = parts[1].parse::<u64>() {
                    return (pages * 4096) as f32 / (1024.0 * 1024.0);
                }
            }
        }
        0.0
    }

    pub fn render(props: DiagnosticsHudProps) -> impl IntoElement {
        let theme = props.theme;
        let fps = props.fps;
        let frame_ms = props.frame_time_ms;
        let rows = props.active_rows;
        let mem = props.memory_mb;

        let fps_color = if fps >= 55.0 {
            theme.success
        } else if fps >= 30.0 {
            theme.warning
        } else {
            theme.danger
        };

        div()
            .flex()
            .items_center()
            .gap(px(10.0))
            .px(px(12.0))
            .py(px(6.0))
            .rounded_full()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border_focus)
            .shadow_lg()
            .text_xs()
            // Metric 1: FPS
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        div()
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .bg(fps_color),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(format!("{:.0} FPS", fps)),
                    ),
            )
            .child(div().text_color(theme.border).child("|"))
            // Metric 2: Frame Latency
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(3.0))
                    .child(div().text_color(theme.text_muted).child("Latence :"))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_primary)
                            .child(format!("{:.2} ms", frame_ms)),
                    ),
            )
            .child(div().text_color(theme.border).child("|"))
            // Metric 3: Virtualized Rows Count
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(3.0))
                    .child(div().text_color(theme.text_muted).child("Lignes :"))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.accent)
                            .child(format!("{}", rows)),
                    ),
            )
            .child(div().text_color(theme.border).child("|"))
            // Metric 4: RSS Memory
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(3.0))
                    .child(div().text_color(theme.text_muted).child("RAM :"))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_primary)
                            .child(format!("{:.1} Mo", mem)),
                    ),
            )
            // Close / Dismiss button
            .child(
                div()
                    .cursor_pointer()
                    .text_color(theme.text_muted)
                    .hover({
                        let hover_text = theme.text_primary;
                        move |s| s.text_color(hover_text)
                    })
                    .child("✕")
                    .when_some(props.on_close, |d, cb| {
                        d.on_mouse_down(MouseButton::Left, move |e, w, cx| cb(e, w, cx))
                    }),
            )
    }
}
