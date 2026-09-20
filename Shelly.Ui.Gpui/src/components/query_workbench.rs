use crate::components::menu::{
    MenuCheckItem, MenuCheckItemProps, MenuDivider, MenuRadioItem, MenuSection, MenuSurface,
};
use crate::components::search_input::SearchInputView;
use crate::components::view_mode_switcher::{ViewModeSwitcher, ViewModeSwitcherProps};
use crate::icons::AppIcon;
use crate::state::package_store::{SourceHealthMap, SourceHealthStatus};
use crate::state::query::{PackageStateFilter, SortMode, SourceScope, WorkbenchBreakpoint};
use crate::state::{PackageSourceKind, PackageViewMode};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkbenchMenu {
    Filters,
    State,
    Sort,
}

pub type SourceToggleHandler = Rc<dyn Fn(PackageSourceKind, &mut Window, &mut App) + 'static>;
pub type StateSelectHandler = Rc<dyn Fn(PackageStateFilter, &mut Window, &mut App) + 'static>;
pub type SortSelectHandler = Rc<dyn Fn(SortMode, &mut Window, &mut App) + 'static>;
pub type ViewModeHandler = Rc<dyn Fn(PackageViewMode, &mut Window, &mut App) + 'static>;
pub type MenuToggleHandler = Rc<dyn Fn(Option<WorkbenchMenu>, &mut Window, &mut App) + 'static>;
pub type SourceRetryHandler = Rc<dyn Fn(PackageSourceKind, &mut Window, &mut App) + 'static>;
pub type ActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct QueryWorkbenchProps<'a> {
    pub search_input: Entity<SearchInputView>,
    pub list_pane_width: f32,
    pub source_scope: SourceScope,
    pub state_filter: PackageStateFilter,
    pub sort_mode: SortMode,
    pub view_mode: PackageViewMode,
    pub active_menu: Option<WorkbenchMenu>,
    pub is_searching: bool,
    pub total_count: usize,
    pub reduce_motion: bool,
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
    pub source_health: &'a SourceHealthMap,
    pub theme: &'a Theme,
    pub on_toggle_source: SourceToggleHandler,
    pub on_select_state: StateSelectHandler,
    pub on_select_sort: SortSelectHandler,
    pub on_select_view_mode: ViewModeHandler,
    pub on_toggle_menu: MenuToggleHandler,
    pub on_retry_source: SourceRetryHandler,
    pub on_clear_filters: ActionHandler,
}

/// Computes the active filter badge count (non-default sources or state).
pub fn compute_filter_badge_count(
    source_scope: &SourceScope,
    state_filter: PackageStateFilter,
    aur_enabled: bool,
    flatpak_enabled: bool,
    appimage_enabled: bool,
) -> usize {
    let mut count = 0;
    if !source_scope.is_all_enabled(aur_enabled, flatpak_enabled, appimage_enabled) {
        count += 1;
    }
    if state_filter != PackageStateFilter::All {
        count += 1;
    }
    count
}

/// Formats the single clean textual summary for active filters, replacing the former chip pile.
pub fn format_active_filter_summary(
    total_count: usize,
    is_searching: bool,
    source_scope: &SourceScope,
    state_filter: PackageStateFilter,
    aur_enabled: bool,
    flatpak_enabled: bool,
    appimage_enabled: bool,
) -> Option<String> {
    let is_all_sources =
        source_scope.is_all_enabled(aur_enabled, flatpak_enabled, appimage_enabled);
    let is_all_states = state_filter == PackageStateFilter::All;
    if is_all_sources && is_all_states {
        return None;
    }

    let mut parts = Vec::new();
    if is_searching {
        parts.push("Searching...".to_string());
    } else {
        let plural = if total_count == 1 {
            "package"
        } else {
            "packages"
        };
        parts.push(format!("Showing {} {}", total_count, plural));
    }

    if !is_all_sources {
        let mut active_srcs = Vec::new();
        if source_scope.alpm {
            active_srcs.push("Arch");
        }
        if aur_enabled && source_scope.aur {
            active_srcs.push("AUR");
        }
        if flatpak_enabled && source_scope.flatpak {
            active_srcs.push("Flatpak");
        }
        if appimage_enabled && source_scope.appimage {
            active_srcs.push("AppImage");
        }
        if active_srcs.is_empty() {
            parts.push("Sources: None".to_string());
        } else {
            parts.push(format!("Sources: {}", active_srcs.join(", ")));
        }
    }

    if !is_all_states {
        parts.push(format!("State: {}", state_filter.label()));
    }

    Some(parts.join("  •  "))
}

pub struct QueryWorkbench;

impl QueryWorkbench {
    pub fn render(props: &QueryWorkbenchProps) -> impl IntoElement {
        let theme = props.theme;
        let breakpoint = WorkbenchBreakpoint::from_width(props.list_pane_width);

        // ── 1. Row 1: Dominant Full-Width Search Input ────────────────────────
        let row1 = div()
            .id("workbench_row1_search")
            .w_full()
            .h(px(40.0))
            .child(props.search_input.clone());

        // ── 2. Filters Popover Setup ──────────────────────────────────────────
        let on_toggle_menu = props.on_toggle_menu.clone();
        let is_filters_open = props.active_menu == Some(WorkbenchMenu::Filters);
        let on_close_filters = {
            let cb = on_toggle_menu.clone();
            Rc::new(move |window: &mut Window, cx: &mut App| {
                cb(None, window, cx);
            })
        };

        let filter_badge_count = compute_filter_badge_count(
            &props.source_scope,
            props.state_filter,
            props.aur_enabled,
            props.flatpak_enabled,
            props.appimage_enabled,
        );

        let filters_btn_text = if filter_badge_count > 0 {
            format!("Filters ({filter_badge_count}) ▾")
        } else {
            "Filters ▾".to_string()
        };

        let focus_border = theme.border_focus;
        let on_toggle_filters_btn = on_toggle_menu.clone();
        let on_toggle_filters_key = on_toggle_menu.clone();

        // Build items for Filters popover
        let mut filter_menu_items = Vec::new();

        // Sources section
        filter_menu_items.push(MenuSection::render("Sources", theme).into_any_element());

        let build_health_badge = |kind: PackageSourceKind| -> (Option<&'static str>, bool) {
            if let Some(h) = props.source_health.get(&kind) {
                match h.status {
                    SourceHealthStatus::Failed => (Some("Failed"), true),
                    SourceHealthStatus::Disabled => (Some("Disabled"), false),
                    SourceHealthStatus::Loading => (Some("Loading..."), false),
                    SourceHealthStatus::Ready => (None, false),
                }
            } else {
                (None, false)
            }
        };

        // ALPM source check item
        {
            let (health_lbl, is_failed) = build_health_badge(PackageSourceKind::Alpm);
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(PackageSourceKind::Alpm, w, a);
                }) as Rc<dyn Fn(&mut Window, &mut App)>)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: "menu_source_alpm".into(),
                    label: "Official / ALPM".into(),
                    is_checked: props.source_scope.alpm,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(PackageSourceKind::Alpm, w, a)),
                    on_retry,
                })
                .into_any_element(),
            );
        }

        // AUR source check item
        if props.aur_enabled {
            let (health_lbl, is_failed) = build_health_badge(PackageSourceKind::Aur);
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(PackageSourceKind::Aur, w, a);
                }) as Rc<dyn Fn(&mut Window, &mut App)>)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: "menu_source_aur".into(),
                    label: "AUR".into(),
                    is_checked: props.source_scope.aur,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(PackageSourceKind::Aur, w, a)),
                    on_retry,
                })
                .into_any_element(),
            );
        }

        // Flatpak source check item
        if props.flatpak_enabled {
            let (health_lbl, is_failed) = build_health_badge(PackageSourceKind::Flatpak);
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(PackageSourceKind::Flatpak, w, a);
                }) as Rc<dyn Fn(&mut Window, &mut App)>)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: "menu_source_flatpak".into(),
                    label: "Flatpak".into(),
                    is_checked: props.source_scope.flatpak,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(PackageSourceKind::Flatpak, w, a)),
                    on_retry,
                })
                .into_any_element(),
            );
        }

        // AppImage source check item
        if props.appimage_enabled {
            let (health_lbl, is_failed) = build_health_badge(PackageSourceKind::AppImage);
            let on_retry = if is_failed {
                let cb = props.on_retry_source.clone();
                Some(Rc::new(move |w: &mut Window, a: &mut App| {
                    cb(PackageSourceKind::AppImage, w, a);
                }) as Rc<dyn Fn(&mut Window, &mut App)>)
            } else {
                None
            };
            let on_toggle = props.on_toggle_source.clone();
            filter_menu_items.push(
                MenuCheckItem::render(MenuCheckItemProps {
                    id: "menu_source_appimage".into(),
                    label: "AppImage".into(),
                    is_checked: props.source_scope.appimage,
                    health_label: health_lbl,
                    is_failed,
                    theme,
                    on_toggle: Rc::new(move |w, a| on_toggle(PackageSourceKind::AppImage, w, a)),
                    on_retry,
                })
                .into_any_element(),
            );
        }

        // Divider
        filter_menu_items.push(MenuDivider::render(theme).into_any_element());

        // Package State section inside Filters menu
        filter_menu_items.push(MenuSection::render("Package State", theme).into_any_element());
        for state in [
            PackageStateFilter::All,
            PackageStateFilter::Installed,
            PackageStateFilter::NotInstalled,
            PackageStateFilter::UpdatesAvailable,
        ] {
            let on_select = props.on_select_state.clone();
            let close_cb = on_close_filters.clone();
            let state_val = state;
            let id_str = match state {
                PackageStateFilter::All => "menu_state_all",
                PackageStateFilter::Installed => "menu_state_installed",
                PackageStateFilter::NotInstalled => "menu_state_not_installed",
                PackageStateFilter::UpdatesAvailable => "menu_state_updates",
            };
            filter_menu_items.push(
                MenuRadioItem::render(
                    id_str.into(),
                    state.label(),
                    props.state_filter == state,
                    theme,
                    Rc::new(move |w, a| {
                        on_select(state_val, w, a);
                        close_cb(w, a);
                    }),
                )
                .into_any_element(),
            );
        }

        let filters_dropdown = if is_filters_open {
            Some(
                deferred(
                    anchored()
                        .anchor(Corner::TopLeft)
                        .offset(point(px(0.0), px(32.0)))
                        .snap_to_window()
                        .child(MenuSurface::render(
                            "filters_menu_surface".into(),
                            theme,
                            px(240.0),
                            props.reduce_motion,
                            on_close_filters,
                            filter_menu_items,
                        )),
                )
                .into_any_element(),
            )
        } else {
            None
        };

        let filters_trigger = div()
            .id("workbench_filters_btn")
            .relative()
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    let next = if is_filters_open {
                        None
                    } else {
                        Some(WorkbenchMenu::Filters)
                    };
                    on_toggle_filters_key(next, window, cx);
                }
            })
            .flex()
            .items_center()
            .h(px(28.0))
            .gap(px(5.0))
            .px(px(10.0))
            .rounded_sm()
            .bg(if is_filters_open {
                theme.bg_surface_active
            } else {
                theme.bg_surface
            })
            .border_1()
            .border_color(if filter_badge_count > 0 || is_filters_open {
                theme.accent
            } else {
                theme.border
            })
            .text_xs()
            .font_weight(if filter_badge_count > 0 {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(if filter_badge_count > 0 {
                theme.accent
            } else {
                theme.text_secondary
            })
            .cursor_pointer()
            .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
            .child(filters_btn_text)
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                let next = if is_filters_open {
                    None
                } else {
                    Some(WorkbenchMenu::Filters)
                };
                on_toggle_filters_btn(next, window, cx);
            })
            .children(filters_dropdown);

        // ── 3. State Dropdown (Wide and Medium Breakpoints) ───────────────────
        let is_state_open = props.active_menu == Some(WorkbenchMenu::State);
        let on_close_state = {
            let cb = on_toggle_menu.clone();
            Rc::new(move |window: &mut Window, cx: &mut App| {
                cb(None, window, cx);
            })
        };

        let state_btn = if breakpoint != WorkbenchBreakpoint::Narrow {
            let mut state_items = Vec::new();
            for state in [
                PackageStateFilter::All,
                PackageStateFilter::Installed,
                PackageStateFilter::NotInstalled,
                PackageStateFilter::UpdatesAvailable,
            ] {
                let on_select = props.on_select_state.clone();
                let close_cb = on_close_state.clone();
                let state_val = state;
                let id_str = match state {
                    PackageStateFilter::All => "state_dropdown_all",
                    PackageStateFilter::Installed => "state_dropdown_installed",
                    PackageStateFilter::NotInstalled => "state_dropdown_not_installed",
                    PackageStateFilter::UpdatesAvailable => "state_dropdown_updates",
                };
                state_items.push(
                    MenuRadioItem::render(
                        id_str.into(),
                        state.label(),
                        props.state_filter == state,
                        theme,
                        Rc::new(move |w, a| {
                            on_select(state_val, w, a);
                            close_cb(w, a);
                        }),
                    )
                    .into_any_element(),
                );
            }

            let state_dropdown = if is_state_open {
                Some(
                    deferred(
                        anchored()
                            .anchor(Corner::TopLeft)
                            .offset(point(px(0.0), px(32.0)))
                            .snap_to_window()
                            .child(MenuSurface::render(
                                "state_menu_surface".into(),
                                theme,
                                px(180.0),
                                props.reduce_motion,
                                on_close_state,
                                state_items,
                            )),
                    )
                    .into_any_element(),
                )
            } else {
                None
            };

            let on_toggle_state_btn = on_toggle_menu.clone();
            let on_toggle_state_key = on_toggle_menu.clone();
            let state_label = format!("State: {} ▾", props.state_filter.label());

            Some(
                div()
                    .id("workbench_state_btn")
                    .relative()
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            let next = if is_state_open {
                                None
                            } else {
                                Some(WorkbenchMenu::State)
                            };
                            on_toggle_state_key(next, window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .h(px(28.0))
                    .gap(px(5.0))
                    .px(px(10.0))
                    .rounded_sm()
                    .bg(if is_state_open {
                        theme.bg_surface_active
                    } else {
                        theme.bg_surface
                    })
                    .border_1()
                    .border_color(if props.state_filter != PackageStateFilter::All {
                        theme.accent
                    } else {
                        theme.border
                    })
                    .text_xs()
                    .text_color(if props.state_filter != PackageStateFilter::All {
                        theme.accent
                    } else {
                        theme.text_secondary
                    })
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
                    .child(state_label)
                    .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                        let next = if is_state_open {
                            None
                        } else {
                            Some(WorkbenchMenu::State)
                        };
                        on_toggle_state_btn(next, window, cx);
                    })
                    .children(state_dropdown),
            )
        } else {
            None
        };

        // ── 4. Sort Dropdown ──────────────────────────────────────────────────
        let is_sort_open = props.active_menu == Some(WorkbenchMenu::Sort);
        let on_close_sort = {
            let cb = on_toggle_menu.clone();
            Rc::new(move |window: &mut Window, cx: &mut App| {
                cb(None, window, cx);
            })
        };

        let mut sort_items = Vec::new();
        for sort in [
            SortMode::Relevance,
            SortMode::NameAsc,
            SortMode::NameDesc,
            SortMode::Source,
            SortMode::InstalledFirst,
            SortMode::UpdatesFirst,
        ] {
            let on_select = props.on_select_sort.clone();
            let close_cb = on_close_sort.clone();
            let sort_val = sort;
            let id_str = match sort {
                SortMode::Relevance => "sort_dropdown_relevance",
                SortMode::NameAsc => "sort_dropdown_name_asc",
                SortMode::NameDesc => "sort_dropdown_name_desc",
                SortMode::Source => "sort_dropdown_source",
                SortMode::InstalledFirst => "sort_dropdown_installed",
                SortMode::UpdatesFirst => "sort_dropdown_updates",
            };
            sort_items.push(
                MenuRadioItem::render(
                    id_str.into(),
                    sort.label(),
                    props.sort_mode == sort,
                    theme,
                    Rc::new(move |w, a| {
                        on_select(sort_val, w, a);
                        close_cb(w, a);
                    }),
                )
                .into_any_element(),
            );
        }

        let sort_dropdown = if is_sort_open {
            Some(
                deferred(
                    anchored()
                        .anchor(Corner::TopLeft)
                        .offset(point(px(0.0), px(32.0)))
                        .snap_to_window()
                        .child(MenuSurface::render(
                            "sort_menu_surface".into(),
                            theme,
                            px(180.0),
                            props.reduce_motion,
                            on_close_sort,
                            sort_items,
                        )),
                )
                .into_any_element(),
            )
        } else {
            None
        };

        let sort_label = if breakpoint == WorkbenchBreakpoint::Narrow {
            match props.sort_mode {
                SortMode::Relevance => "Sort: Rel ▾",
                SortMode::NameAsc => "Sort: A–Z ▾",
                SortMode::NameDesc => "Sort: Z–A ▾",
                SortMode::Source => "Sort: Src ▾",
                SortMode::InstalledFirst => "Sort: Inst ▾",
                SortMode::UpdatesFirst => "Sort: Upd ▾",
            }
        } else {
            match props.sort_mode {
                SortMode::Relevance => "Sort: Relevance ▾",
                SortMode::NameAsc => "Sort: Name (A-Z) ▾",
                SortMode::NameDesc => "Sort: Name (Z-A) ▾",
                SortMode::Source => "Sort: Source ▾",
                SortMode::InstalledFirst => "Sort: Installed First ▾",
                SortMode::UpdatesFirst => "Sort: Updates First ▾",
            }
        };

        let on_toggle_sort_btn = on_toggle_menu.clone();
        let on_toggle_sort_key = on_toggle_menu.clone();

        let sort_btn = div()
            .id("workbench_sort_btn")
            .relative()
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    let next = if is_sort_open {
                        None
                    } else {
                        Some(WorkbenchMenu::Sort)
                    };
                    on_toggle_sort_key(next, window, cx);
                }
            })
            .flex()
            .items_center()
            .h(px(28.0))
            .gap(px(5.0))
            .px(px(10.0))
            .rounded_sm()
            .bg(if is_sort_open {
                theme.bg_surface_active
            } else {
                theme.bg_surface
            })
            .border_1()
            .border_color(if props.sort_mode != SortMode::Relevance {
                theme.accent
            } else {
                theme.border
            })
            .text_xs()
            .text_color(if props.sort_mode != SortMode::Relevance {
                theme.accent
            } else {
                theme.text_secondary
            })
            .cursor_pointer()
            .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
            .child(sort_label)
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                let next = if is_sort_open {
                    None
                } else {
                    Some(WorkbenchMenu::Sort)
                };
                on_toggle_sort_btn(next, window, cx);
            })
            .children(sort_dropdown);

        // ── 5. View Mode Switcher & Count ─────────────────────────────────────
        let view_switcher = ViewModeSwitcher::render(ViewModeSwitcherProps {
            current_mode: props.view_mode,
            theme,
            on_select_mode: props.on_select_view_mode.clone(),
        });

        let summary_text = format_active_filter_summary(
            props.total_count,
            props.is_searching,
            &props.source_scope,
            props.state_filter,
            props.aur_enabled,
            props.flatpak_enabled,
            props.appimage_enabled,
        );
        let has_active_filters = summary_text.is_some();

        let status_badge = if !has_active_filters {
            if props.is_searching {
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
            }
        } else {
            None
        };

        // ── 6. Row 2 Composition ──────────────────────────────────────────────
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
                    .child(filters_trigger),
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

        // ── 7. Row 3: Active Filter Textual Summary ───────────────────────────
        let row3 = summary_text.map(|summary| {
            let on_clear = props.on_clear_filters.clone();
            let on_clear_key = on_clear.clone();

            let clear_btn = div()
                .id("workbench_clear_filters_btn")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(focus_border))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        on_clear_key(window, cx);
                    }
                })
                .flex()
                .items_center()
                .gap(px(4.0))
                .px(px(6.0))
                .py(px(2.0))
                .rounded_sm()
                .cursor_pointer()
                .text_xs()
                .text_color(theme.accent)
                .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.accent_hover))
                .child(
                    svg()
                        .path(AppIcon::Close.path())
                        .size(px(10.0))
                        .text_color(theme.accent),
                )
                .child("Clear filters")
                .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                    on_clear(window, cx);
                });

            div()
                .id("workbench_row3_active_filters")
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .w_full()
                .px(px(4.0))
                .py(px(2.0))
                .child(
                    div()
                        .id("workbench_textual_summary")
                        .text_xs()
                        .text_color(theme.text_secondary)
                        .child(summary),
                )
                .child(clear_btn)
        });

        // ── 8. Final Container Assembly ───────────────────────────────────────
        div()
            .id("query_workbench_container")
            .w_full()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .p(px(8.0))
            .bg(theme.bg_sidebar)
            .border_b_1()
            .border_color(theme.border)
            .child(row1)
            .child(row2)
            .children(row3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_format_active_filter_summary_default_returns_none() {
        let scope = SourceScope {
            alpm: true,
            aur: true,
            flatpak: true,
            appimage: true,
        };
        let summary = format_active_filter_summary(
            150,
            false,
            &scope,
            PackageStateFilter::All,
            true,
            true,
            true,
        );
        assert!(summary.is_none());
    }

    #[test]
    fn test_format_active_filter_summary_source_subset() {
        let scope = SourceScope {
            alpm: true,
            aur: true,
            flatpak: false,
            appimage: false,
        };
        let summary = format_active_filter_summary(
            42,
            false,
            &scope,
            PackageStateFilter::All,
            true,
            true,
            true,
        );
        assert_eq!(
            summary,
            Some("Showing 42 packages  •  Sources: Arch, AUR".to_string())
        );
    }

    #[test]
    fn test_format_active_filter_summary_state_filter_only() {
        let scope = SourceScope {
            alpm: true,
            aur: true,
            flatpak: true,
            appimage: true,
        };
        let summary = format_active_filter_summary(
            12,
            false,
            &scope,
            PackageStateFilter::Installed,
            true,
            true,
            true,
        );
        assert_eq!(
            summary,
            Some("Showing 12 packages  •  State: Installed".to_string())
        );
    }

    #[test]
    fn test_format_active_filter_summary_both_sources_and_state() {
        let scope = SourceScope {
            alpm: true,
            aur: false,
            flatpak: false,
            appimage: false,
        };
        let summary = format_active_filter_summary(
            1,
            false,
            &scope,
            PackageStateFilter::UpdatesAvailable,
            true,
            true,
            true,
        );
        assert_eq!(
            summary,
            Some("Showing 1 package  •  Sources: Arch  •  State: Updates Available".to_string())
        );
    }

    #[test]
    fn test_format_active_filter_summary_searching_state() {
        let scope = SourceScope {
            alpm: true,
            aur: false,
            flatpak: false,
            appimage: false,
        };
        let summary = format_active_filter_summary(
            0,
            true,
            &scope,
            PackageStateFilter::All,
            true,
            true,
            true,
        );
        assert_eq!(summary, Some("Searching...  •  Sources: Arch".to_string()));
    }

    #[test]
    fn test_compute_filter_badge_count() {
        let all_scope = SourceScope {
            alpm: true,
            aur: true,
            flatpak: true,
            appimage: true,
        };
        assert_eq!(
            compute_filter_badge_count(&all_scope, PackageStateFilter::All, true, true, true),
            0
        );

        let subset_scope = SourceScope {
            alpm: true,
            aur: false,
            flatpak: false,
            appimage: false,
        };
        assert_eq!(
            compute_filter_badge_count(&subset_scope, PackageStateFilter::All, true, true, true),
            1
        );
        assert_eq!(
            compute_filter_badge_count(&all_scope, PackageStateFilter::Installed, true, true, true),
            1
        );
        assert_eq!(
            compute_filter_badge_count(
                &subset_scope,
                PackageStateFilter::Installed,
                true,
                true,
                true
            ),
            2
        );
    }
}
