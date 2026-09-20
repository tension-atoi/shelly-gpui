use crate::backend::models::UnifiedPackage;
use crate::components::status_pill::StatusPill;
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
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
            .h(px(72.0))
            .w_full()
            .flex()
            .flex_col()
            .justify_between()
            .px_3()
            .py_2()
            .rounded_md()
            .border_1()
            .border_color(border_color)
            .bg(bg_color)
            .cursor_pointer()
            .overflow_hidden()
            // Barre d'accentuation latérale pour le paquet sélectionné
            .when(is_selected, |el| el.border_l_4().border_color(theme.accent))
            // Rétroaction immédiate au survol de la souris
            .when(!is_selected, |el| {
                let hover_bg = theme.bg_surface_hover;
                let hover_border = theme.border_focus;
                el.hover(move |s| s.bg(hover_bg).border_color(hover_border))
            })
            // Ligne 1 : Nom du paquet + Version
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_sm()
                            .text_color(if is_selected {
                                theme.accent
                            } else {
                                theme.text_primary
                            })
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(pkg.name.clone()),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_muted)
                            .child(pkg.version.clone()),
                    ),
            )
            // Ligne 2 : Badges de source et d'état (alignés sans collision)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(StatusPill::source_badge(&pkg.source_type, theme))
                    .child(StatusPill::installed_pill(pkg.is_installed, theme)),
            )
            // Ligne 3 : Description aérée tronquée proprement à 1 ligne pour uniformité stricte
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(if pkg.description.is_empty() {
                        "Aucune description disponible pour ce paquet.".to_string()
                    } else {
                        pkg.description.clone()
                    }),
            )
    }
}
