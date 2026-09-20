use crate::components::view_mode_switcher::{ViewModeSwitcher, ViewModeSwitcherProps};
use crate::icons::AppIcon;
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
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
    pub theme: &'a Theme,
    pub on_select_filter: SourceFilterHandler,
    pub on_select_view_mode: ViewModeHandler,
}

pub struct UnifiedSearch;

impl UnifiedSearch {
    fn render_pill(
        filter: SourceFilter,
        active_filter: SourceFilter,
        accent_color: Option<Rgba>,
        theme: &Theme,
        on_select: SourceFilterHandler,
    ) -> impl IntoElement {
        let is_active = filter == active_filter;
        let label = filter.label();

        let element_id = match filter {
            SourceFilter::All => "source_filter_all",
            SourceFilter::Alpm => "source_filter_alpm",
            SourceFilter::Aur => "source_filter_aur",
            SourceFilter::Flatpak => "source_filter_flatpak",
            SourceFilter::AppImage => "source_filter_appimage",
        };

        let focus_border = theme.border_focus;
        let on_select_key = on_select.clone();

        let base = div()
            .id(element_id)
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_select_key(filter, window, cx);
                }
            })
            .px(px(8.0))
            .py(px(3.0))
            .rounded_md()
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            });

        let pill = if is_active {
            if let Some(accent) = accent_color {
                base.bg(accent)
                    .text_color(theme.bg_app)
                    .border_1()
                    .border_color(accent)
            } else {
                base.bg(theme.accent)
                    .text_color(theme.bg_app)
                    .border_1()
                    .border_color(theme.accent)
            }
        } else {
            let hover_bg = theme.bg_surface_hover;
            base.bg(theme.bg_surface)
                .text_color(theme.text_secondary)
                .border_1()
                .border_color(theme.border)
                .hover(move |s| s.bg(hover_bg).text_color(theme.text_primary))
        };

        pill.child(label)
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_select(filter, window, cx);
            })
    }

    pub fn render_filter_bar(props: &UnifiedSearchProps) -> impl IntoElement {
        let theme = props.theme;
        let active = props.active_filter;
        let on_select = props.on_select_filter.clone();

        let view_switcher = ViewModeSwitcher::render(ViewModeSwitcherProps {
            current_mode: props.view_mode,
            theme,
            on_select_mode: props.on_select_view_mode.clone(),
        });

        let mut pills = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(Self::render_pill(
                SourceFilter::All,
                active,
                None,
                theme,
                on_select.clone(),
            ))
            .child(Self::render_pill(
                SourceFilter::Alpm,
                active,
                Some(theme.badge_alpm),
                theme,
                on_select.clone(),
            ));

        if props.aur_enabled {
            pills = pills.child(Self::render_pill(
                SourceFilter::Aur,
                active,
                Some(theme.badge_aur),
                theme,
                on_select.clone(),
            ));
        }

        if props.flatpak_enabled {
            pills = pills.child(Self::render_pill(
                SourceFilter::Flatpak,
                active,
                Some(theme.badge_flatpak),
                theme,
                on_select.clone(),
            ));
        }

        if props.appimage_enabled {
            pills = pills.child(Self::render_pill(
                SourceFilter::AppImage,
                active,
                Some(theme.badge_appimage),
                theme,
                on_select,
            ));
        }

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .bg(theme.bg_surface)
            .border_b_1()
            .border_color(theme.border)
            .child(pills)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(if props.is_searching {
                        div()
                            .text_xs()
                            .text_color(theme.accent)
                            .child("Searching...")
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
                svg()
                    .path(AppIcon::Search.path())
                    .size_8()
                    .text_color(theme.accent)
                    .mb_3(),
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
