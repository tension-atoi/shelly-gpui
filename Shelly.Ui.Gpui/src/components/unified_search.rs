use crate::state::{PackageViewMode, SourceFilter};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type SourceFilterHandler = Rc<dyn Fn(SourceFilter, &mut Window, &mut App) + 'static>;
pub type ViewModeHandler = Rc<dyn Fn(PackageViewMode, &mut Window, &mut App) + 'static>;

pub struct UnifiedSearchProps<'a> {
    pub active_filter: SourceFilter,
    pub is_searching: bool,
    pub total_count: usize,
    pub view_mode: PackageViewMode,
    pub theme: &'a Theme,
    pub on_select_filter: SourceFilterHandler,
    pub on_select_view_mode: ViewModeHandler,
}

pub struct UnifiedSearch;

impl UnifiedSearch {
    fn render_pill(
        filter: SourceFilter,
        label: &'static str,
        active_filter: SourceFilter,
        badge_color: Option<Rgba>,
        theme: &Theme,
        on_select: SourceFilterHandler,
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

        let is_cards = props.view_mode == PackageViewMode::Cards;
        let is_table = props.view_mode == PackageViewMode::Table;
        let on_cards = props.on_select_view_mode.clone();
        let on_table = props.on_select_view_mode.clone();

        let view_switcher = div()
            .flex()
            .items_center()
            .bg(theme.bg_app)
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .p(px(2.0))
            .gap(px(2.0))
            .child(
                div()
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded_sm()
                    .cursor_pointer()
                    .text_xs()
                    .font_weight(if is_cards {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .bg(if is_cards {
                        theme.bg_surface_active
                    } else {
                        theme.bg_app
                    })
                    .text_color(if is_cards {
                        theme.accent
                    } else {
                        theme.text_muted
                    })
                    .hover(move |s| s.text_color(theme.text_primary))
                    .child(format!(
                        "{} {}",
                        PackageViewMode::Cards.icon(),
                        PackageViewMode::Cards.label()
                    ))
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        on_cards(PackageViewMode::Cards, window, cx);
                    }),
            )
            .child(
                div()
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded_sm()
                    .cursor_pointer()
                    .text_xs()
                    .font_weight(if is_table {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .bg(if is_table {
                        theme.bg_surface_active
                    } else {
                        theme.bg_app
                    })
                    .text_color(if is_table {
                        theme.accent
                    } else {
                        theme.text_muted
                    })
                    .hover(move |s| s.text_color(theme.text_primary))
                    .child(format!(
                        "{} {}",
                        PackageViewMode::Table.icon(),
                        PackageViewMode::Table.label()
                    ))
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        on_table(PackageViewMode::Table, window, cx);
                    }),
            );

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
                            .child("⚡ Searching...")
                    } else if props.total_count > 0 {
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("{} results", props.total_count))
                    } else {
                        div()
                    })
                    .child(view_switcher),
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
                    .child("Unified Search in Shelly"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .max_w(px(460.0))
                    .mb_4()
                    .child(
                        "Type a package or application name to search across Arch repositories, the AUR, Flatpak, and AppImages.",
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
                    .child("• No network requests are dispatched on empty input")
                    .child("• Source pills filter results by distribution backend")
                    .child("• Use Up / Down arrows to navigate through the virtualized results"),
            )
    }
}
