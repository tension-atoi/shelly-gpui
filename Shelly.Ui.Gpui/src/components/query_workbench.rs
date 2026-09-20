use crate::components::menu::{
    MenuCheckItem, MenuCheckItemProps, MenuRadioItem, MenuSection, MenuSurface,
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

pub struct QueryWorkbench;

impl QueryWorkbench {
    fn render_quick_pill(
        kind: PackageSourceKind,
        label: &'static str,
        id_str: &'static str,
        is_selected: bool,
        accent_color: Option<Rgba>,
        theme: &Theme,
        on_toggle: SourceToggleHandler,
    ) -> impl IntoElement {
        let focus_border = theme.border_focus;
        let on_toggle_key = on_toggle.clone();

        let base = div()
            .id(id_str)
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_toggle_key(kind, window, cx);
                }
            })
            .px(px(8.0))
            .py(px(2.5))
            .rounded_md()
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_selected {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            });

        let pill = if is_selected {
            let color = accent_color.unwrap_or(theme.accent);
            base.bg(color)
                .text_color(theme.bg_app)
                .border_1()
                .border_color(color)
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
                on_toggle(kind, window, cx);
            })
    }

    pub fn render(props: &QueryWorkbenchProps) -> impl IntoElement {
        let theme = props.theme;
        let breakpoint = WorkbenchBreakpoint::from_width(props.list_pane_width);

        // ── 1. Row 1: Dominant Full-Width Search Input ────────────────────────
        let input_width = (props.list_pane_width - 16.0).max(200.0);
        let row1 = div()
            .id("workbench_row1_search")
            .w(px(input_width))
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

        let filter_badge_count = {
            let mut count = 0;
            if !props.source_scope.is_all_enabled(
                props.aur_enabled,
                props.flatpak_enabled,
                props.appimage_enabled,
            ) {
                count += 1;
            }
            if props.state_filter != PackageStateFilter::All {
                count += 1;
            }
            count
        };

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
                        .offset(point(px(0.0), px(28.0)))
                        .snap_to_window()
                        .child(MenuSurface::render(
                            "filters_menu_surface".into(),
                            theme,
                            px(240.0),
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
            .gap(px(4.0))
            .px(px(8.0))
            .py(px(2.5))
            .rounded_md()
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

        // ── 3. Quick Source Pills (Wide Breakpoint Only) ──────────────────────
        let mut quick_source_pills = Vec::new();
        if breakpoint == WorkbenchBreakpoint::Wide {
            quick_source_pills.push(
                Self::render_quick_pill(
                    PackageSourceKind::Alpm,
                    "ALPM",
                    "workbench_quick_source_alpm",
                    props.source_scope.alpm,
                    Some(theme.badge_alpm),
                    theme,
                    props.on_toggle_source.clone(),
                )
                .into_any_element(),
            );

            if props.aur_enabled {
                quick_source_pills.push(
                    Self::render_quick_pill(
                        PackageSourceKind::Aur,
                        "AUR",
                        "workbench_quick_source_aur",
                        props.source_scope.aur,
                        Some(theme.badge_aur),
                        theme,
                        props.on_toggle_source.clone(),
                    )
                    .into_any_element(),
                );
            }

            if props.flatpak_enabled {
                quick_source_pills.push(
                    Self::render_quick_pill(
                        PackageSourceKind::Flatpak,
                        "Flatpak",
                        "workbench_quick_source_flatpak",
                        props.source_scope.flatpak,
                        Some(theme.badge_flatpak),
                        theme,
                        props.on_toggle_source.clone(),
                    )
                    .into_any_element(),
                );
            }

            if props.appimage_enabled {
                quick_source_pills.push(
                    Self::render_quick_pill(
                        PackageSourceKind::AppImage,
                        "AppImage",
                        "workbench_quick_source_appimage",
                        props.source_scope.appimage,
                        Some(theme.badge_appimage),
                        theme,
                        props.on_toggle_source.clone(),
                    )
                    .into_any_element(),
                );
            }
        }

        // ── 4. State Dropdown (Wide and Medium Breakpoints) ───────────────────
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
                            .offset(point(px(0.0), px(28.0)))
                            .snap_to_window()
                            .child(MenuSurface::render(
                                "state_menu_surface".into(),
                                theme,
                                px(180.0),
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
                    .gap(px(4.0))
                    .px(px(8.0))
                    .py(px(2.5))
                    .rounded_md()
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

        // ── 5. Sort Dropdown ──────────────────────────────────────────────────
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
                        .offset(point(px(0.0), px(28.0)))
                        .snap_to_window()
                        .child(MenuSurface::render(
                            "sort_menu_surface".into(),
                            theme,
                            px(180.0),
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
            .gap(px(4.0))
            .px(px(8.0))
            .py(px(2.5))
            .rounded_md()
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

        // ── 6. View Mode Switcher & Count ─────────────────────────────────────
        let view_switcher = ViewModeSwitcher::render(ViewModeSwitcherProps {
            current_mode: props.view_mode,
            theme,
            on_select_mode: props.on_select_view_mode.clone(),
        });

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

        // ── 7. Row 2 Composition ──────────────────────────────────────────────
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
                    .child(filters_trigger)
                    .children(quick_source_pills),
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

        // ── 8. Row 3: Active Filter Chips & Clear Filters ─────────────────────
        let is_all_sources = props.source_scope.is_all_enabled(
            props.aur_enabled,
            props.flatpak_enabled,
            props.appimage_enabled,
        );
        let is_all_states = props.state_filter == PackageStateFilter::All;
        let has_active_filters = !is_all_sources || !is_all_states;

        let row3 = if has_active_filters {
            let mut active_chips = Vec::new();

            // Source filter chips if not all enabled
            if !is_all_sources {
                if props.source_scope.alpm {
                    let on_toggle = props.on_toggle_source.clone();
                    active_chips.push(
                        div()
                            .id("active_chip_alpm")
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .px(px(6.0))
                            .py(px(1.5))
                            .rounded_sm()
                            .bg(theme.bg_surface)
                            .border_1()
                            .border_color(theme.badge_alpm)
                            .text_xs()
                            .text_color(theme.badge_alpm)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.bg_surface_hover))
                            .child("Official ×")
                            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                                on_toggle(PackageSourceKind::Alpm, window, cx);
                            })
                            .into_any_element(),
                    );
                }
                if props.aur_enabled && props.source_scope.aur {
                    let on_toggle = props.on_toggle_source.clone();
                    active_chips.push(
                        div()
                            .id("active_chip_aur")
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .px(px(6.0))
                            .py(px(1.5))
                            .rounded_sm()
                            .bg(theme.bg_surface)
                            .border_1()
                            .border_color(theme.badge_aur)
                            .text_xs()
                            .text_color(theme.badge_aur)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.bg_surface_hover))
                            .child("AUR ×")
                            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                                on_toggle(PackageSourceKind::Aur, window, cx);
                            })
                            .into_any_element(),
                    );
                }
                if props.flatpak_enabled && props.source_scope.flatpak {
                    let on_toggle = props.on_toggle_source.clone();
                    active_chips.push(
                        div()
                            .id("active_chip_flatpak")
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .px(px(6.0))
                            .py(px(1.5))
                            .rounded_sm()
                            .bg(theme.bg_surface)
                            .border_1()
                            .border_color(theme.badge_flatpak)
                            .text_xs()
                            .text_color(theme.badge_flatpak)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.bg_surface_hover))
                            .child("Flatpak ×")
                            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                                on_toggle(PackageSourceKind::Flatpak, window, cx);
                            })
                            .into_any_element(),
                    );
                }
                if props.appimage_enabled && props.source_scope.appimage {
                    let on_toggle = props.on_toggle_source.clone();
                    active_chips.push(
                        div()
                            .id("active_chip_appimage")
                            .flex()
                            .items_center()
                            .gap(px(2.0))
                            .px(px(6.0))
                            .py(px(1.5))
                            .rounded_sm()
                            .bg(theme.bg_surface)
                            .border_1()
                            .border_color(theme.badge_appimage)
                            .text_xs()
                            .text_color(theme.badge_appimage)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.bg_surface_hover))
                            .child("AppImage ×")
                            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                                on_toggle(PackageSourceKind::AppImage, window, cx);
                            })
                            .into_any_element(),
                    );
                }
            }

            // State filter chip if not All
            if !is_all_states {
                let on_select = props.on_select_state.clone();
                let state_lbl = format!("{} ×", props.state_filter.label());
                active_chips.push(
                    div()
                        .id("active_chip_state")
                        .flex()
                        .items_center()
                        .gap(px(2.0))
                        .px(px(6.0))
                        .py(px(1.5))
                        .rounded_sm()
                        .bg(theme.bg_surface)
                        .border_1()
                        .border_color(theme.accent)
                        .text_xs()
                        .text_color(theme.accent)
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.bg_surface_hover))
                        .child(state_lbl)
                        .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                            on_select(PackageStateFilter::All, window, cx);
                        })
                        .into_any_element(),
                );
            }

            // Clear filters button
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
                .py(px(1.5))
                .rounded_sm()
                .cursor_pointer()
                .text_xs()
                .text_color(theme.accent)
                .hover(|s| s.bg(theme.bg_surface_hover))
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

            Some(
                div()
                    .id("workbench_row3_active_filters")
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(6.0))
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .flex_wrap()
                            .gap(px(4.0))
                            .children(active_chips),
                    )
                    .child(clear_btn),
            )
        } else {
            None
        };

        // ── 9. Final Container Assembly ───────────────────────────────────────
        div()
            .id("query_workbench_container")
            .w(px(props.list_pane_width))
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
