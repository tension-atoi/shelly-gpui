use crate::backend::models::UnifiedPackage;
use crate::components::package_card::{PackageCard, PackageCardProps};
use crate::components::package_table::PackageTable;
use crate::components::unified_search::UnifiedSearch;
use crate::state::console::{ConsoleEvent, ConsoleModel};
use crate::state::{
    canonical_install_command, AppSession, NavDestination, PackageStore, PackageViewMode,
    SourceFilter, ToastCenter, ToastKind,
};
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use crate::views::inspector::{PackageInspectorProps, PackageInspectorView};
use gpui::*;
use gpui::{uniform_list, UniformListScrollHandle};
use std::rc::Rc;

pub type PackageMutationHandler =
    Rc<dyn Fn(&UnifiedPackage, bool /* is_install */, &mut Window, &mut App) + 'static>;
pub type SystemUpgradeHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitterDragState {
    pub start_width: f32,
    pub start_pointer_x: f32,
}

/// Calcul pur et robuste de la largeur du panneau dérivé des deltas de pointeur
/// Garantit un minimum invariant pour la liste et l'inspecteur à toute largeur de fenêtre
pub fn compute_splitter_width(
    drag_start_width: f32,
    drag_start_pointer_x: f32,
    current_pointer_x: f32,
    window_width: f32,
) -> f32 {
    let delta_x = current_pointer_x - drag_start_pointer_x;
    let requested_width = drag_start_width + delta_x;
    let dynamic_list_max = (window_width
        - UiMetrics::SIDEBAR_EXPANDED
        - UiMetrics::SPLITTER_WIDTH
        - UiMetrics::INSPECTOR_MIN_WIDTH)
        .min(700.0);
    requested_width.clamp(
        UiMetrics::LIST_MIN_WIDTH,
        dynamic_list_max.max(UiMetrics::LIST_MIN_WIDTH),
    )
}

pub struct PackageWorkstationView {
    pub session: Entity<AppSession>,
    pub store: Entity<PackageStore>,
    pub console: Entity<ConsoleModel>,
    pub toast_center: Entity<ToastCenter>,
    pub list_pane_width: f32,
    pub drag_state: Option<SplitterDragState>,
    pub scroll_handle: UniformListScrollHandle,
    pub on_mutation: Option<PackageMutationHandler>,
    pub on_upgrade_all: Option<SystemUpgradeHandler>,
    pub copy_cmd_feedback: bool,
    pub is_loading_pkgbuild: bool,
    pub reduce_motion: bool,
    pub theme: Theme,
    pub compact: bool,
}

impl PackageWorkstationView {
    pub fn new(
        session: Entity<AppSession>,
        store: Entity<PackageStore>,
        console: Entity<ConsoleModel>,
        toast_center: Entity<ToastCenter>,
        theme: Theme,
        reduce_motion: bool,
        compact: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.subscribe(&session, |_this, _emitter, _event, cx| {
            cx.notify();
        })
        .detach();

        cx.subscribe(&store, |_this, _emitter, _event, cx| {
            cx.notify();
        })
        .detach();

        cx.subscribe(&console, |_this, _emitter, event, cx| {
            match event {
                ConsoleEvent::OperationStarted(_) | ConsoleEvent::OperationFinished(_) => {
                    cx.notify();
                }
                _ => {}
            }
        })
        .detach();

        Self {
            session,
            store,
            console,
            toast_center,
            list_pane_width: 460.0,
            drag_state: None,
            scroll_handle: UniformListScrollHandle::new(),
            on_mutation: None,
            on_upgrade_all: None,
            copy_cmd_feedback: false,
            is_loading_pkgbuild: false,
            reduce_motion,
            theme,
            compact,
        }
    }

    pub fn set_on_mutation(&mut self, handler: PackageMutationHandler) {
        self.on_mutation = Some(handler);
    }

    pub fn set_on_upgrade_all(&mut self, handler: SystemUpgradeHandler) {
        self.on_upgrade_all = Some(handler);
    }

    pub fn on_pointer_move(
        &mut self,
        event: &MouseMoveEvent,
        window_width: f32,
        cx: &mut Context<Self>,
    ) {
        if let Some(drag) = self.drag_state {
            let current_x = event.position.x.to_f64() as f32;
            let new_width =
                compute_splitter_width(drag.start_width, drag.start_pointer_x, current_x, window_width);
            if (new_width - self.list_pane_width).abs() >= 1.0 {
                self.list_pane_width = new_width;
                cx.notify();
            }
        }
    }

    pub fn set_compact(&mut self, compact: bool, cx: &mut Context<Self>) {
        if self.compact != compact {
            self.compact = compact;
            cx.notify();
        }
    }

    pub fn set_reduce_motion(&mut self, reduce_motion: bool, cx: &mut Context<Self>) {
        if self.reduce_motion != reduce_motion {
            self.reduce_motion = reduce_motion;
            cx.notify();
        }
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.theme = theme;
        cx.notify();
    }

    pub fn on_pointer_up(&mut self, cx: &mut Context<Self>) {
        if self.drag_state.is_some() {
            self.drag_state = None;
            cx.notify();
        }
    }
}

impl Render for PackageWorkstationView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let entity = cx.entity().clone();
        let is_busy = self.console.read(cx).is_running();

        let (
            destination,
            source_filter,
            is_searching,
            selected_key,
            view_mode,
            active_tab,
            search_query,
            reduce_motion,
            inspector_tab_epoch,
        ) = {
            let s = self.session.read(cx);
            (
                s.destination,
                s.source_filter,
                s.is_searching,
                s.selected_package_key.clone(),
                s.view_mode,
                s.inspector_tab,
                s.search_query.clone(),
                self.reduce_motion,
                s.inspector_tab_epoch,
            )
        };

        let packages: std::sync::Arc<[UnifiedPackage]> = {
            let store = self.store.read(cx);
            match destination {
                NavDestination::Browse => std::sync::Arc::clone(&store.active_results),
                NavDestination::Installed => std::sync::Arc::clone(&store.installed_packages),
                NavDestination::Updates => std::sync::Arc::clone(&store.updates_packages),
                _ => std::sync::Arc::from([]),
            }
        };

        let selected_pkg = selected_key
            .as_ref()
            .and_then(|key| packages.iter().find(|p| &p.key() == key).cloned());

        let entity_filter = entity.clone();
        let entity_mode = entity.clone();
        let entity_select = entity.clone();
        let entity_split = entity.clone();
        let entity_move = entity.clone();
        let entity_up = entity.clone();

        // ── 1. Top bar de filtrage et de vue ────────────────────────────────
        let filter_bar = UnifiedSearch::render_filter_bar(
            &crate::components::unified_search::UnifiedSearchProps {
                active_filter: source_filter,
                is_searching,
                total_count: packages.len(),
                view_mode,
                theme: &theme,
                on_select_filter: {
                    let on_filter = entity_filter.clone();
                    Rc::new(move |filter, _w, cx| {
                        on_filter.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_source_filter(filter, cx);
                            });
                        });
                    })
                },
                on_select_view_mode: {
                    let on_mode = entity_mode.clone();
                    Rc::new(move |mode, _w, cx| {
                        on_mode.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_view_mode(mode, cx);
                            });
                        });
                    })
                },
            },
        );

        let mode_switcher = {
            let on_mode = entity_mode.clone();
            crate::components::view_mode_switcher::ViewModeSwitcher::render(
                crate::components::view_mode_switcher::ViewModeSwitcherProps {
                    current_mode: view_mode,
                    theme: &theme,
                    on_select_mode: Rc::new(move |mode, _w, cx| {
                        on_mode.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_view_mode(mode, cx);
                            });
                        });
                    }),
                },
            )
        };

        let top_bar = match destination {
            crate::state::NavDestination::Browse => {
                let entity_input = entity.clone();
                let search_input = div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2p5()
                    .bg(theme.bg_sidebar)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        svg()
                            .path(crate::icons::AppIcon::Search.path())
                            .size_4()
                            .text_color(theme.text_muted),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(theme.text_primary)
                            .child(if search_query.is_empty() {
                                div()
                                    .text_color(theme.text_muted)
                                    .child("Search packages (e.g. ripgrep, firefox)...")
                            } else {
                                div().child(search_query.clone())
                            }),
                    )
                    .child(if is_searching {
                        if self.reduce_motion {
                            div()
                                .text_xs()
                                .text_color(theme.accent)
                                .child("Searching...")
                                .into_any_element()
                        } else {
                            div()
                                .id("searching_indicator")
                                .text_xs()
                                .text_color(theme.accent)
                                .child("Searching...")
                                .with_animation(
                                    ("search_pulse", 0usize),
                                    Animation::new(std::time::Duration::from_millis(800))
                                        .repeat()
                                        .with_easing(gpui::pulsating_between(0.4, 1.0)),
                                    |el, delta| el.opacity(delta),
                                )
                                .into_any_element()
                        }
                    } else if !packages.is_empty() {
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("{} results", packages.len()))
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    })
                    .on_key_down(move |event, _w, cx| {
                        let key = event.keystroke.key.as_str();
                        entity_input.update(cx, |view, cx| {
                            let mut cur = view.session.read(cx).search_query.clone();
                            if key == "backspace" {
                                cur.pop();
                                view.session.update(cx, |s, cx| s.set_search_query(cur, cx));
                            } else if key == "escape" {
                                view.session
                                    .update(cx, |s, cx| s.set_search_query(String::new(), cx));
                            } else if event.keystroke.key.chars().count() == 1 {
                                cur.push_str(key);
                                view.session.update(cx, |s, cx| s.set_search_query(cur, cx));
                            }
                        });
                    });

                div()
                    .flex()
                    .flex_col()
                    .child(search_input)
                    .child(filter_bar)
            }
            crate::state::NavDestination::Installed => div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .bg(theme.bg_surface)
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .text_color(theme.text_primary)
                        .child(
                            svg()
                                .path(crate::icons::AppIcon::Installed.path())
                                .size_4()
                                .text_color(theme.accent),
                        )
                        .child("Locally Installed Packages"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(format!("{} packages", packages.len())),
                        )
                        .child(mode_switcher),
                ),
            crate::state::NavDestination::Updates => {
                let on_upgrade_cb = self.on_upgrade_all.clone();
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .bg(theme.bg_surface)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .font_weight(FontWeight::BOLD)
                            .text_sm()
                            .text_color(theme.text_primary)
                            .child(
                                svg()
                                    .path(crate::icons::AppIcon::Updates.path())
                                    .size_4()
                                    .text_color(theme.accent),
                            )
                            .child("Available System Updates"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(if is_busy {
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme.bg_surface_active)
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text_muted)
                                    .child("Upgrade All")
                            } else {
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(theme.accent)
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.bg_app)
                                    .cursor_pointer()
                                    .hover({
                                        let h = theme.accent_hover;
                                        move |s| s.bg(h)
                                    })
                                    .child("Upgrade All")
                                    .on_mouse_down(MouseButton::Left, move |_e, w, cx| {
                                        if let Some(ref cb) = on_upgrade_cb {
                                            cb(w, cx);
                                        }
                                    })
                            })
                            .child(mode_switcher),
                    )
            }
            _ => div(),
        };

        // ── 2. Corps de la liste (Cards vs Table) ───────────────────────────
        let is_browse_empty =
            destination == crate::state::NavDestination::Browse && search_query.trim().is_empty();

        let list_body = if is_browse_empty {
            div()
                .flex_1()
                .child(UnifiedSearch::render_empty_discovery(&theme))
                .into_any_element()
        } else if packages.is_empty() {
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
                        .text_sm()
                        .text_color(theme.text_muted)
                        .child(if is_searching {
                            "Recherche en cours..."
                        } else {
                            "Aucun paquet correspondant."
                        }),
                )
                .into_any_element()
        } else {
            let package_count = packages.len();
            let selected_name = selected_pkg.as_ref().map(|p| p.name.clone());
            let list_theme = theme;

            if view_mode == PackageViewMode::Table {
                let row_h = if self.compact {
                    UiMetrics::ROW_HEIGHT_COMPACT
                } else {
                    UiMetrics::ROW_HEIGHT_NORMAL
                };
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .child(PackageTable::render_header(&theme))
                    .child(
                        div().flex_1().h_full().overflow_hidden().child(
                            uniform_list("package-list-table", package_count, {
                                let packages = packages.clone();
                                let selected_name = selected_name.clone();
                                let is_compact = self.compact;

                                move |range, _window, _cx| {
                                    range
                                        .map(|idx| {
                                            let pkg = &packages[idx];
                                            let is_selected = selected_name
                                                .as_ref()
                                                .map(|n| n == &pkg.name)
                                                .unwrap_or(false);
                                            let pkg_key = pkg.key();
                                            let on_select = entity_select.clone();

                                            div()
                                                .h(px(row_h))
                                                .child(PackageTable::render_row(
                                                    pkg,
                                                    is_selected,
                                                    &list_theme,
                                                    is_compact,
                                                ))
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    move |_e, _w, cx| {
                                                        let key = pkg_key.clone();
                                                        on_select.update(cx, |view, cx| {
                                                            view.session.update(cx, |s, cx| {
                                                                s.select_package(Some(key), cx);
                                                            });
                                                        });
                                                    },
                                                )
                                        })
                                        .collect()
                                }
                            })
                            .h_full()
                            .track_scroll(self.scroll_handle.clone()),
                        ),
                    )
                    .into_any_element()
            } else {
                let card_wrapper_h = if self.compact {
                    UiMetrics::CARD_WRAPPER_COMPACT
                } else {
                    UiMetrics::CARD_WRAPPER_NORMAL
                };
                let is_compact = self.compact;
                uniform_list("package-list-surface", package_count, {
                    let packages = packages.clone();
                    let selected_name = selected_name.clone();

                    move |range, _window, _cx| {
                        range
                            .map(|idx| {
                                let pkg = &packages[idx];
                                let is_selected = selected_name
                                    .as_ref()
                                    .map(|n| n == &pkg.name)
                                    .unwrap_or(false);
                                let pkg_key = pkg.key();
                                let on_select = entity_select.clone();

                                div()
                                    .h(px(card_wrapper_h))
                                    .px_3()
                                    .py_1()
                                    .child(PackageCard::render(PackageCardProps {
                                        package: pkg,
                                        is_selected,
                                        theme: &list_theme,
                                        compact: is_compact,
                                    }))
                                    .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                                        let key = pkg_key.clone();
                                        on_select.update(cx, |view, cx| {
                                            view.session.update(cx, |s, cx| {
                                                s.select_package(Some(key), cx);
                                            });
                                        });
                                    })
                            })
                            .collect()
                    }
                })
                .h_full()
                .track_scroll(self.scroll_handle.clone())
                .into_any_element()
            }
        };

        let list_pane = div()
            .flex()
            .flex_col()
            .w(px(self.list_pane_width))
            .h_full()
            .border_r_1()
            .border_color(theme.border)
            .child(top_bar)
            .child(div().flex_1().overflow_hidden().child(list_body));

        // ── 3. Splitter draggable isolé ─────────────────────────────────────
        let is_resizing = self.drag_state.is_some();
        let splitter = div()
            .id("workstation_splitter")
            .w(px(5.0))
            .h_full()
            .bg(if is_resizing {
                theme.accent
            } else {
                theme.border
            })
            .cursor_col_resize()
            .hover({
                let h = theme.accent;
                move |s| s.bg(h)
            })
            .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
                let pointer_x = event.position.x.to_f64() as f32;
                entity_split.update(cx, |view, cx| {
                    view.drag_state = Some(SplitterDragState {
                        start_width: view.list_pane_width,
                        start_pointer_x: pointer_x,
                    });
                    cx.notify();
                });
            });

        // ── 4. Panneau d'inspection sémantique ──────────────────────────────
        let alpm_details = selected_key
            .as_ref()
            .and_then(|k| self.store.read(cx).get_cached_details(k).cloned());

        let cached_pkgbuild = selected_pkg.as_ref().and_then(|p| {
            if p.source_type == "AUR" {
                self.store.read(cx).get_cached_pkgbuild(&p.name)
            } else {
                None
            }
        });

        let selected_pkg_clone = selected_pkg.clone();
        let on_mutation_cb = self.on_mutation.clone();
        let on_mutation_cb_rm = self.on_mutation.clone();
        let entity_tab = entity.clone();
        let entity_nav_dep = entity.clone();
        let tc_entity = self.toast_center.clone();

        let details_pane = div().flex_1().h_full().overflow_hidden().child(
            PackageInspectorView::render_with_motion(
                PackageInspectorProps {
                    package: selected_pkg.as_ref(),
                    alpm_details: alpm_details.as_ref(),
                    pkgbuild: cached_pkgbuild,
                    is_loading_pkgbuild: self.is_loading_pkgbuild,
                    active_tab,
                    theme: &theme,
                    is_busy,
                    copy_feedback: self.copy_cmd_feedback,
                    on_select_tab: Rc::new(move |tab, _w, cx| {
                        entity_tab.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_inspector_tab(tab, cx);
                            });
                        });
                    }),
                    on_install: selected_pkg_clone.as_ref().and_then(|p| {
                        on_mutation_cb.as_ref().map(|cb| {
                            let p_c = p.clone();
                            let cb_c = cb.clone();
                            Rc::new(
                                move |_e: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                    cb_c(&p_c, true, window, cx);
                                },
                            )
                                as Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>
                        })
                    }),
                    on_remove: selected_pkg_clone.as_ref().and_then(|p| {
                        on_mutation_cb_rm.as_ref().map(|cb| {
                            let p_c = p.clone();
                            let cb_c = cb.clone();
                            Rc::new(
                                move |_e: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                    cb_c(&p_c, false, window, cx);
                                },
                            )
                                as Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>
                        })
                    }),
                    on_copy_install_cmd: selected_pkg_clone.as_ref().and_then(|p| {
                        canonical_install_command(p).map(|cmd| {
                            let tc = tc_entity.clone();
                            let rm = reduce_motion;
                            Rc::new(move |_w: &mut Window, cx: &mut App| {
                                cx.write_to_clipboard(ClipboardItem::new_string(cmd.clone()));
                                tc.update(cx, |center, cx| {
                                    center.post(
                                        ToastKind::Success,
                                        "Command copied",
                                        cmd.clone(),
                                        None,
                                        rm,
                                        cx,
                                    );
                                });
                            }) as Rc<dyn Fn(&mut Window, &mut App)>
                        })
                    }),
                    on_copy_pkgbuild: cached_pkgbuild.cloned().map(|content| {
                        let tc = tc_entity.clone();
                        let rm = reduce_motion;
                        Rc::new(move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(content.clone()));
                            tc.update(cx, |center, cx| {
                                center.post(
                                    ToastKind::Success,
                                    "PKGBUILD copied",
                                    "The PKGBUILD recipe has been copied to your clipboard.",
                                    None,
                                    rm,
                                    cx,
                                );
                            });
                        }) as Rc<dyn Fn(&mut Window, &mut App)>
                    }),
                    on_navigate_package: Some(Rc::new(move |pkg_name, _w, cx| {
                        entity_nav_dep.update(cx, |view, cx| {
                            view.session.update(cx, |s, cx| {
                                s.set_destination(crate::state::NavDestination::Browse, cx);
                                s.set_source_filter(SourceFilter::All, cx);
                                s.set_search_query(pkg_name.clone(), cx);
                                s.select_package(None, cx);
                            });
                        });
                    })),
                },
                reduce_motion,
                inspector_tab_epoch,
            ),
        );

        // ── 5. Conteneur racine de la station de travail absorbant les moves ──
        div()
            .id("package_workstation_root")
            .flex()
            .flex_row()
            .size_full()
            .overflow_hidden()
            .on_mouse_move(move |event, window, cx| {
                let window_width = window.window_bounds().get_bounds().size.width / px(1.0);
                entity_move.update(cx, |view, cx| {
                    view.on_pointer_move(event, window_width, cx);
                });
            })
            .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                entity_up.update(cx, |view, cx| {
                    view.on_pointer_up(cx);
                });
            })
            .child(list_pane)
            .child(splitter)
            .child(details_pane)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_splitter_delta_forward() {
        let width = compute_splitter_width(460.0, 650.0, 700.0, 1280.0);
        assert_eq!(width, 510.0);
    }

    #[test]
    fn test_splitter_delta_reverse() {
        let width = compute_splitter_width(460.0, 650.0, 600.0, 1280.0);
        assert_eq!(width, 410.0);
    }

    #[test]
    fn test_splitter_clamp_minimum() {
        let width = compute_splitter_width(460.0, 650.0, 100.0, 1280.0);
        assert_eq!(width, 280.0, "Must clamp to 280 minimum");
    }

    #[test]
    fn test_splitter_clamp_maximum() {
        let width = compute_splitter_width(460.0, 650.0, 1200.0, 1280.0);
        assert_eq!(width, 700.0, "Must clamp to 700 maximum at 1280px");
    }

    #[test]
    fn test_splitter_sidebar_offset_independence() {
        // Sidebar collapsed (56px) vs expanded (190px)
        let collapsed_start_x = 56.0 + 460.0; // 516
        let collapsed_current_x = 56.0 + 520.0; // 576
        let width_collapsed = compute_splitter_width(460.0, collapsed_start_x, collapsed_current_x, 1280.0);

        let expanded_start_x = 190.0 + 460.0; // 650
        let expanded_current_x = 190.0 + 520.0; // 710
        let width_expanded = compute_splitter_width(460.0, expanded_start_x, expanded_current_x, 1280.0);

        assert_eq!(width_collapsed, 520.0);
        assert_eq!(width_expanded, 520.0);
        assert_eq!(
            width_collapsed, width_expanded,
            "Splitter width calculation must be completely independent of sidebar width"
        );
    }

    #[test]
    fn test_splitter_guarantees_inspector_minimum_across_reference_viewports() {
        // Reference size 1: 1024x680
        // max list = (1024 - 190 - 5 - 320).min(700) = 509
        let width_1024 = compute_splitter_width(460.0, 650.0, 1200.0, 1024.0);
        assert_eq!(width_1024, 509.0);
        let inspector_1024 = 1024.0 - UiMetrics::SIDEBAR_EXPANDED - UiMetrics::SPLITTER_WIDTH - width_1024;
        assert!(
            inspector_1024 >= UiMetrics::INSPECTOR_MIN_WIDTH,
            "Inspector must have at least 320px at 1024px viewport (got {})",
            inspector_1024
        );

        // Reference size 2: 1280x840
        let width_1280 = compute_splitter_width(460.0, 650.0, 1200.0, 1280.0);
        assert_eq!(width_1280, 700.0);
        let inspector_1280 = 1280.0 - UiMetrics::SIDEBAR_EXPANDED - UiMetrics::SPLITTER_WIDTH - width_1280;
        assert!(
            inspector_1280 >= UiMetrics::INSPECTOR_MIN_WIDTH,
            "Inspector must have at least 320px at 1280px viewport (got {})",
            inspector_1280
        );

        // Reference size 3: 1600x1000
        let width_1600 = compute_splitter_width(460.0, 650.0, 1200.0, 1600.0);
        assert_eq!(width_1600, 700.0);
        let inspector_1600 = 1600.0 - UiMetrics::SIDEBAR_EXPANDED - UiMetrics::SPLITTER_WIDTH - width_1600;
        assert!(
            inspector_1600 >= UiMetrics::INSPECTOR_MIN_WIDTH,
            "Inspector must have at least 320px at 1600px viewport (got {})",
            inspector_1600
        );
    }
}
