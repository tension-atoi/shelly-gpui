use crate::state::SourceFilter;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub struct UnifiedSearchProps<'a> {
    pub active_filter: SourceFilter,
    pub is_searching: bool,
    pub total_count: usize,
    pub theme: &'a Theme,
    pub on_select_filter: Rc<dyn Fn(SourceFilter, &mut Window, &mut App) + 'static>,
}

pub struct UnifiedSearch;

impl UnifiedSearch {
    fn render_pill(
        filter: SourceFilter,
        label: &'static str,
        active_filter: SourceFilter,
        badge_color: Option<Rgba>,
        theme: &Theme,
        on_select: Rc<dyn Fn(SourceFilter, &mut Window, &mut App) + 'static>,
    ) -> impl IntoElement {
        let is_active = active_filter == filter;
        let base = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .px(px(10.0))
            .py(px(4.0))
            .rounded_full()
            .cursor_pointer()
            .text_size(px(12.0));

        let styled_pill = if is_active {
            base.bg(theme.bg_surface_active)
                .text_color(theme.accent)
                .border_1()
                .border_color(theme.border_focus)
                .font_weight(FontWeight::SEMIBOLD)
        } else {
            let hover_bg = theme.bg_surface_hover;
            let hover_text = theme.text_primary;
            base.bg(theme.bg_surface)
                .text_color(theme.text_secondary)
                .border_1()
                .border_color(theme.border)
                .hover(move |s| s.bg(hover_bg).text_color(hover_text))
        };

        let badge_bg = badge_color.unwrap_or(theme.text_muted);

        styled_pill
            .child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(badge_bg))
            .child(label)
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_select(filter, window, cx);
            })
    }

    pub fn render_filter_bar(props: &UnifiedSearchProps) -> impl IntoElement {
        let theme = props.theme;
        let active = props.active_filter;
        let on_select = props.on_select_filter.clone();

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .bg(theme.bg_surface)
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(Self::render_pill(
                        SourceFilter::All,
                        SourceFilter::All.label(),
                        active,
                        None,
                        theme,
                        on_select.clone(),
                    ))
                    .child(Self::render_pill(
                        SourceFilter::Alpm,
                        SourceFilter::Alpm.label(),
                        active,
                        Some(theme.badge_alpm),
                        theme,
                        on_select.clone(),
                    ))
                    .child(Self::render_pill(
                        SourceFilter::Aur,
                        SourceFilter::Aur.label(),
                        active,
                        Some(theme.badge_aur),
                        theme,
                        on_select.clone(),
                    ))
                    .child(Self::render_pill(
                        SourceFilter::Flatpak,
                        SourceFilter::Flatpak.label(),
                        active,
                        Some(theme.badge_flatpak),
                        theme,
                        on_select.clone(),
                    ))
                    .child(Self::render_pill(
                        SourceFilter::AppImage,
                        SourceFilter::AppImage.label(),
                        active,
                        Some(theme.badge_appimage),
                        theme,
                        on_select,
                    )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(if props.is_searching {
                        div()
                            .text_xs()
                            .text_color(theme.accent)
                            .child("⚡ Recherche en cours...")
                    } else if props.total_count > 0 {
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("{} résultats", props.total_count))
                    } else {
                        div()
                    }),
            )
    }

    /// Rendu de l'état de découverte lorsque la barre de recherche est vide
    pub fn render_empty_discovery(theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .size_full()
            .p_8()
            .text_center()
            .child(
                div()
                    .text_3xl()
                    .mb_3()
                    .child("🔍"),
            )
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .mb_2()
                    .child("Recherche unifiée dans Shelly"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .max_w(px(460.0))
                    .mb_4()
                    .child(
                        "Tapez le nom d'un logiciel ou paquet pour explorer simultanément les dépôts officiels Arch, l'AUR et Flatpak.",
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_3()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("• Aucune requête réseau superflue n'est émise à vide")
                    .child("• Les filtres ci-dessus permettent de restreindre la recherche à une source précise")
                    .child("• Naviguez au clavier avec les flèches ↑ / ↓ pour faire défiler la sélection"),
            )
    }
}
