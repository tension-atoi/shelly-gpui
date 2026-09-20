use crate::components::search_input::SearchInputView;
use crate::components::view_mode_switcher::{ViewModeSwitcher, ViewModeSwitcherProps};
use crate::icons::AppIcon;
use crate::state::query::{PackageStateFilter, SortMode, SourceScope, WorkbenchBreakpoint};
use crate::state::{PackageViewMode, SourceFilter};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type SourceFilterHandler = Rc<dyn Fn(SourceFilter, &mut Window, &mut App) + 'static>;
pub type ViewModeHandler = Rc<dyn Fn(PackageViewMode, &mut Window, &mut App) + 'static>;
pub type ActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct QueryWorkbenchProps<'a> {
    pub search_input: Entity<SearchInputView>,
    pub list_pane_width: f32,
    pub source_filter: SourceFilter,
    pub source_scope: SourceScope,
    pub state_filter: PackageStateFilter,
    pub sort_mode: SortMode,
    pub view_mode: PackageViewMode,
    pub is_searching: bool,
    pub total_count: usize,
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
    pub theme: &'a Theme,
    pub on_select_filter: SourceFilterHandler,
    pub on_cycle_sort: ActionHandler,
    pub on_cycle_state_filter: ActionHandler,
    pub on_select_view_mode: ViewModeHandler,
    pub on_reset_query: ActionHandler,
}

pub struct QueryWorkbench;

impl QueryWorkbench {
    fn render_pill(
        filter: SourceFilter,
        active_filter: SourceFilter,
        accent_color: Option<Rgba>,
        compact_label: bool,
        theme: &Theme,
        on_select: SourceFilterHandler,
    ) -> impl IntoElement {
        let is_active = filter == active_filter;
        let label = if compact_label {
            match filter {
                SourceFilter::All => "All",
                SourceFilter::Alpm => "ALPM",
                SourceFilter::Aur => "AUR",
                SourceFilter::Flatpak => "FP",
                SourceFilter::AppImage => "AI",
            }
        } else {
            filter.label()
        };

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
            .py(px(2.5))
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

    pub fn render(props: &QueryWorkbenchProps) -> impl IntoElement {
        let theme = props.theme;
        let breakpoint = WorkbenchBreakpoint::from_width(props.list_pane_width);
        let active_filter = props.source_filter;
        let on_select = props.on_select_filter.clone();

        // ── 1. Row 1: Search Input ───────────────────────────────────────────
        let row1 = div().w_full().child(props.search_input.clone());

        // ── 2. Row 2: Query Controls & View Mode ──────────────────────────────
        let view_switcher = ViewModeSwitcher::render(ViewModeSwitcherProps {
            current_mode: props.view_mode,
            theme,
            on_select_mode: props.on_select_view_mode.clone(),
        });

        // Source selector depending on responsive breakpoint
        let is_compact_labels = breakpoint == WorkbenchBreakpoint::Medium;
        let source_controls = if breakpoint == WorkbenchBreakpoint::Narrow {
            // Narrow mode: compact button that displays current source filter and cycles
            let on_click = on_select.clone();
            let next_filter = match active_filter {
                SourceFilter::All => SourceFilter::Alpm,
                SourceFilter::Alpm => {
                    if props.aur_enabled {
                        SourceFilter::Aur
                    } else if props.flatpak_enabled {
                        SourceFilter::Flatpak
                    } else if props.appimage_enabled {
                        SourceFilter::AppImage
                    } else {
                        SourceFilter::All
                    }
                }
                SourceFilter::Aur => {
                    if props.flatpak_enabled {
                        SourceFilter::Flatpak
                    } else if props.appimage_enabled {
                        SourceFilter::AppImage
                    } else {
                        SourceFilter::All
                    }
                }
                SourceFilter::Flatpak => {
                    if props.appimage_enabled {
                        SourceFilter::AppImage
                    } else {
                        SourceFilter::All
                    }
                }
                SourceFilter::AppImage => SourceFilter::All,
            };
            let focus_border = theme.border_focus;
            let on_key = on_click.clone();
            div()
                .id("workbench_narrow_source_btn")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(focus_border))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        on_key(next_filter, window, cx);
                    }
                })
                .flex()
                .items_center()
                .gap(px(4.0))
                .px(px(8.0))
                .py(px(2.5))
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(theme.border)
                .text_xs()
                .text_color(theme.text_secondary)
                .cursor_pointer()
                .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
                .child(format!("Src: {}", active_filter.label()))
                .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                    on_click(next_filter, window, cx);
                })
                .into_any_element()
        } else {
            // Wide and Medium modes: pills
            let mut pills = div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(Self::render_pill(
                    SourceFilter::All,
                    active_filter,
                    None,
                    is_compact_labels,
                    theme,
                    on_select.clone(),
                ))
                .child(Self::render_pill(
                    SourceFilter::Alpm,
                    active_filter,
                    Some(theme.badge_alpm),
                    is_compact_labels,
                    theme,
                    on_select.clone(),
                ));

            if props.aur_enabled {
                pills = pills.child(Self::render_pill(
                    SourceFilter::Aur,
                    active_filter,
                    Some(theme.badge_aur),
                    is_compact_labels,
                    theme,
                    on_select.clone(),
                ));
            }

            if props.flatpak_enabled {
                pills = pills.child(Self::render_pill(
                    SourceFilter::Flatpak,
                    active_filter,
                    Some(theme.badge_flatpak),
                    is_compact_labels,
                    theme,
                    on_select.clone(),
                ));
            }

            if props.appimage_enabled {
                pills = pills.child(Self::render_pill(
                    SourceFilter::AppImage,
                    active_filter,
                    Some(theme.badge_appimage),
                    is_compact_labels,
                    theme,
                    on_select,
                ));
            }

            pills.into_any_element()
        };

        // Sort mode cycle button
        let on_sort_click = props.on_cycle_sort.clone();
        let on_sort_key = on_sort_click.clone();
        let focus_border = theme.border_focus;
        let sort_label = if breakpoint == WorkbenchBreakpoint::Narrow {
            match props.sort_mode {
                SortMode::Relevance => "Rel",
                SortMode::NameAsc => "A–Z",
                SortMode::NameDesc => "Z–A",
                SortMode::Source => "Src",
                SortMode::InstalledFirst => "Inst",
                SortMode::UpdatesFirst => "Upd",
            }
        } else {
            props.sort_mode.label()
        };

        let sort_btn = div()
            .id("workbench_sort_btn")
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_sort_key(window, cx);
                }
            })
            .flex()
            .items_center()
            .gap(px(4.0))
            .px(px(8.0))
            .py(px(2.5))
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .text_xs()
            .text_color(theme.text_secondary)
            .cursor_pointer()
            .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
            .child(format!("Sort: {sort_label}"))
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_sort_click(window, cx);
            });

        // State filter cycle button (only shown in Wide / Medium mode)
        let state_btn = if breakpoint != WorkbenchBreakpoint::Narrow {
            let on_state_click = props.on_cycle_state_filter.clone();
            let on_state_key = on_state_click.clone();
            let state_label = props.state_filter.label();
            Some(
                div()
                    .id("workbench_state_btn")
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_state_key(window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(8.0))
                    .py(px(2.5))
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
                    .child(state_label)
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        on_state_click(window, cx);
                    }),
            )
        } else {
            None
        };

        // Reset button when filters are active
        let has_active_filters = active_filter != SourceFilter::All
            || props.sort_mode != SortMode::Relevance
            || props.state_filter != PackageStateFilter::All;

        let reset_btn = if has_active_filters {
            let on_reset_click = props.on_reset_query.clone();
            let on_reset_key = on_reset_click.clone();
            let reset_bg = Rgba {
                a: 0.15,
                ..theme.accent
            };
            let reset_hover_bg = Rgba {
                a: 0.25,
                ..theme.accent
            };
            let summary_text = format!(
                "{} · {} · {}",
                props.source_scope.summary_label(),
                props.state_filter.label(),
                props.sort_mode.label()
            );
            Some(
                div()
                    .id("workbench_reset_filters_btn")
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_reset_key(window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .px(px(6.0))
                    .py(px(2.5))
                    .rounded_md()
                    .bg(reset_bg)
                    .border_1()
                    .border_color(theme.accent)
                    .text_xs()
                    .text_color(theme.accent)
                    .cursor_pointer()
                    .hover(move |s| s.bg(reset_hover_bg))
                    .child(
                        svg()
                            .path(AppIcon::Close.path())
                            .size(px(10.0))
                            .text_color(theme.accent),
                    )
                    .child(if breakpoint == WorkbenchBreakpoint::Wide {
                        format!("Reset: {summary_text}")
                    } else {
                        "Reset".to_string()
                    })
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        on_reset_click(window, cx);
                    }),
            )
        } else {
            None
        };

        let status_badge = if props.is_searching {
            Some(
                div()
                    .text_xs()
                    .text_color(theme.accent)
                    .child("Searching..."),
            )
        } else if props.total_count > 0 {
            Some(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(format!("{} pkgs", props.total_count)),
            )
        } else {
            None
        };

        let row2 = div()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(6.0))
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(source_controls)
                    .children(reset_btn),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .children(status_badge)
                    .children(state_btn)
                    .child(sort_btn)
                    .child(view_switcher),
            );

        div()
            .id("query_workbench_container")
            .flex()
            .flex_col()
            .gap(px(6.0))
            .p(px(8.0))
            .bg(theme.bg_sidebar)
            .border_b_1()
            .border_color(theme.border)
            .child(row1)
            .child(row2)
    }
}
