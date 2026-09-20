use crate::theme::Theme;
use gpui::*;

pub struct StatusPill;

impl StatusPill {

    pub fn source_badge(source: &str, theme: &Theme) -> impl IntoElement {
        let (bg, text) = match source.to_uppercase().as_str() {
            "ALPM" => (theme.badge_alpm, theme.bg_app),
            "AUR" => (theme.badge_aur, theme.bg_app),
            "FLATPAK" => (theme.badge_flatpak, theme.bg_app),
            "APPIMAGE" => (theme.badge_appimage, theme.bg_app),
            _ => (theme.border, theme.text_primary),
        };

        div()
            .px_1p5()
            .py_0p5()
            .rounded_sm()
            .bg(bg)
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(text)
            .child(source.to_string())
    }

    pub fn installed_pill(is_installed: bool, theme: &Theme) -> impl IntoElement {
        if is_installed {
            div()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(theme.success)
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.bg_app)
                .child("Installé")
        } else {
            div()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(theme.border)
                .text_xs()
                .font_weight(FontWeight::NORMAL)
                .text_color(theme.text_muted)
                .child("Disponible")
        }
    }
}
