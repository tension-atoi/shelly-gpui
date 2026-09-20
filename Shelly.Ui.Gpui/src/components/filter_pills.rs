use crate::models::SourceFilter;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub struct FilterPillsProps<'a> {
    pub active_filter: SourceFilter,
    pub total_count: usize,
    pub official_count: usize,
    pub aur_count: usize,
    pub flatpak_count: usize,
    pub theme: &'a Theme,
    pub on_select_filter: Rc<dyn Fn(SourceFilter, &mut Window, &mut App) + 'static>,
}

pub struct FilterPills;

impl FilterPills {
    fn render_pill(
        filter: SourceFilter,
        label: &'static str,
        count: usize,
        badge_color: Option<Rgba>,
        active_filter: SourceFilter,
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

        let badge_bg = if let Some(c) = badge_color {
            c
        } else {
            theme.text_muted
        };

        styled_pill
            .child(label)
            .child(
                div()
                    .bg(badge_bg)
                    .text_color(theme.bg_app)
                    .text_size(px(10.0))
                    .font_weight(FontWeight::BOLD)
                    .rounded_full()
                    .px(px(5.0))
                    .py(px(0.5))
                    .child(format!("{}", count)),
            )
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_select(filter, window, cx);
            })
    }

    pub fn render(props: FilterPillsProps) -> impl IntoElement {
        let theme = props.theme;
        let on_select = props.on_select_filter.clone();

        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(Self::render_pill(
                SourceFilter::All,
                "Tous",
                props.total_count,
                None,
                props.active_filter,
                theme,
                on_select.clone(),
            ))
            .child(Self::render_pill(
                SourceFilter::Official,
                "Officiels",
                props.official_count,
                Some(theme.badge_alpm),
                props.active_filter,
                theme,
                on_select.clone(),
            ))
            .child(Self::render_pill(
                SourceFilter::Aur,
                "AUR",
                props.aur_count,
                Some(theme.badge_aur),
                props.active_filter,
                theme,
                on_select.clone(),
            ))
            .child(Self::render_pill(
                SourceFilter::Flatpak,
                "Flatpak",
                props.flatpak_count,
                Some(theme.badge_flatpak),
                props.active_filter,
                theme,
                on_select,
            ))
    }
}
