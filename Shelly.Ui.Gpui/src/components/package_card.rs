use crate::backend::models::UnifiedPackage;
use crate::components::status_pill::StatusPill;
use crate::theme::Theme;
use gpui::*;

pub struct PackageCardProps<'a> {
    pub package: &'a UnifiedPackage,
    pub is_selected: bool,
    pub theme: &'a Theme,
}

pub struct PackageCard;

impl PackageCard {
    pub fn render(props: PackageCardProps) -> impl IntoElement {
        let theme = props.theme;
        let pkg = props.package;
        let is_selected = props.is_selected;

        let bg_color = if is_selected {
            theme.bg_surface_active
        } else {
            theme.bg_surface
        };

        let border_color = if is_selected {
            theme.border_focus
        } else {
            theme.border
        };

        div()
            .flex()
            .flex_col()
            .p_3()
            .mb_2()
            .rounded_md()
            .border_1()
            .border_color(border_color)
            .bg(bg_color)
            .cursor_pointer()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_sm()
                                    .text_color(theme.text_primary)
                                    .child(pkg.name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child(pkg.version.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(StatusPill::source_badge(&pkg.source_type, theme))
                            .child(StatusPill::installed_pill(pkg.is_installed, theme)),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .line_clamp(2)
                    .child(if pkg.description.is_empty() {
                        "Aucune description disponible.".to_string()
                    } else {
                        pkg.description.clone()
                    }),
            )
    }
}
