use crate::backend::client::ShellyClient;
use crate::backend::models::{ArchNewsItem, UnifiedPackage};
use crate::backend::process::LogStreamEvent;
use crate::components::toast_overlay::{ToastOverlay, ToastOverlayProps};
use crate::config::{ConfigManager, GpuiUiConfig, ShellySettings};
use crate::state::package_store::SourceHealth;
use crate::state::{
    AppSession, ConsoleEvent, ConsoleModel, InspectorTab, MotionDurations, NavDestination,
    PackageKey, PackageSourceKind, PackageStore, PackageStoreEvent, SessionEvent, ToastAction,
    ToastCenter, ToastKind,
};
use crate::theme::Theme;
use crate::views::news::{NewsView, NewsViewProps};
use crate::views::operation_console::OperationConsoleView;
use crate::views::package_workstation::{PackageWorkstationConfig, PackageWorkstationView};
use crate::views::settings::{SettingsView, SettingsViewProps};
use crate::views::sidebar::SidebarView;
use gpui::ScrollStrategy;
use gpui::*;
use std::rc::Rc;
use tokio::sync::mpsc;

enum MutationAction {
    Install { is_aur: bool, is_flatpak: bool },
    Remove { is_flatpak: bool },
    UpgradeSystem,
}

pub struct WorkspaceView {
    pub session: Entity<AppSession>,
    pub store: Entity<PackageStore>,
    pub console: Entity<ConsoleModel>,
    pub toast_center: Entity<ToastCenter>,
    pub workstation: Entity<PackageWorkstationView>,
    pub sidebar: Entity<SidebarView>,
    pub console_view: Entity<OperationConsoleView>,
    pub settings: SettingsView,
    pub shelly_settings: ShellySettings,
    pub gpui_config: GpuiUiConfig,
    pub theme: Theme,
    pub search_focus: FocusHandle,
    pub search_input_buffer: String,
    pub search_debounce_task: Option<Task<()>>,
    pub news: Vec<ArchNewsItem>,
    pub is_loading_news: bool,
    pub motion_policy: crate::state::motion::MotionPolicy,
}

impl WorkspaceView {
    pub fn with_config(
        shelly_settings: ShellySettings,
        gpui_config: GpuiUiConfig,
        cx: &mut Context<Self>,
    ) -> Self {
        let theme = if gpui_config.dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        };

        let client = ShellyClient::new(None);
        let session = cx.new(|_cx| {
            let mut s = AppSession::with_initial_tab(gpui_config.last_selected_tab);
            if gpui_config.compact_view {
                s.sidebar_collapsed = true;
            }
            s
        });
        let store = cx.new(|_cx| PackageStore::new(client));
        let console = cx.new(|_cx| {
            let mut c = ConsoleModel::new();
            c.is_open = gpui_config.log_drawer_open;
            c.auto_open = gpui_config.log_drawer_open;
            c
        });
        let toast_center = cx.new(|_cx| ToastCenter::new());

        let reduce_motion = gpui_config.reduce_motion;
        let compact = gpui_config.compact_view;
        let aur_enabled = shelly_settings.aur_enabled;
        let flatpak_enabled = shelly_settings.flat_pack_enabled;
        let appimage_enabled = shelly_settings.app_image_enabled;

        session.update(cx, |s, cx| {
            s.clamp_source_scope(aur_enabled, flatpak_enabled, appimage_enabled, cx);
        });

        let workstation = cx.new(|cx| {
            PackageWorkstationView::new(
                session.clone(),
                store.clone(),
                console.clone(),
                toast_center.clone(),
                PackageWorkstationConfig {
                    theme,
                    reduce_motion,
                    compact,
                    aur_enabled,
                    flatpak_enabled,
                    appimage_enabled,
                },
                cx,
            )
        });

        let sidebar =
            cx.new(|cx| SidebarView::new(session.clone(), store.clone(), theme, reduce_motion, cx));

        let console_view = cx.new(|cx| {
            OperationConsoleView::new(
                console.clone(),
                toast_center.clone(),
                gpui_config.log_drawer_height,
                theme,
                reduce_motion,
                cx,
            )
        });

        let settings = SettingsView::new(shelly_settings.clone(), gpui_config.clone());

        // Abonnements réactifs aux événements de session
        cx.subscribe(&session, |this, _emitter, event, cx| match event {
            SessionEvent::DestinationChanged(dest) => {
                log::debug!("Navigation vers la destination : {:?}", dest);
                this.on_destination_changed(*dest, cx);
                cx.notify();
            }
            SessionEvent::SourceScopeChanged(scope) => {
                log::debug!("Changement de scope de source : {:?}", scope);
                let query = this.session.read(cx).search_query.clone();
                if !query.trim().is_empty() {
                    this.execute_search(query, cx);
                }
                cx.notify();
            }
            SessionEvent::StateFilterChanged(_) | SessionEvent::SortModeChanged(_) => {
                cx.notify();
            }
            SessionEvent::SearchQueryChanged(q) => {
                log::debug!("Requête de recherche mise à jour : '{}'", q);
                this.workstation.update(cx, |ws, cx| {
                    ws.search_input.update(cx, |si, cx| {
                        if si.text() != q {
                            si.set_text(q.clone(), cx);
                        }
                    });
                });
                this.on_search_input(q.clone(), cx);
                cx.notify();
            }
            SessionEvent::SearchingStateChanged(searching) => {
                let searching_val = *searching;
                this.workstation.update(cx, |ws, cx| {
                    ws.search_input.update(cx, |si, cx| {
                        si.set_is_searching(searching_val, cx);
                    });
                });
                cx.notify();
            }
            SessionEvent::PackageSelected(opt_key) => {
                log::debug!("Paquet sélectionné : {:?}", opt_key);
                if let Some(key) = opt_key {
                    this.ensure_package_details(key.clone(), cx);
                    if this.session.read(cx).inspector_tab == InspectorTab::FilesBuild
                        && key.source == PackageSourceKind::Aur
                    {
                        this.ensure_pkgbuild(key.name.clone(), cx);
                    }
                }
                cx.notify();
            }
            SessionEvent::SidebarToggled(_) => {
                cx.notify();
            }
            SessionEvent::ViewModeChanged(_) => {
                cx.notify();
            }
            SessionEvent::InspectorTabChanged(tab) => {
                if *tab == InspectorTab::FilesBuild {
                    if let Some(ref key) = this.session.read(cx).selected_package_key {
                        if key.source == PackageSourceKind::Aur {
                            this.ensure_pkgbuild(key.name.clone(), cx);
                        }
                    }
                }
                cx.notify();
            }
        })
        .detach();

        // Abonnement réactif à SearchInput de PackageWorkstationView
        let search_input_ent = workstation.read(cx).search_input.clone();
        cx.subscribe(&search_input_ent, |this, _emitter, event, cx| match event {
            crate::components::search_input::SearchEvent::Changed(query) => {
                this.on_search_input(query.clone(), cx);
            }
            crate::components::search_input::SearchEvent::Submitted(query) => {
                this.execute_search(query.clone(), cx);
            }
            crate::components::search_input::SearchEvent::Cleared => {
                this.on_search_input(String::new(), cx);
            }
        })
        .detach();

        // Abonnements réactifs typés aux événements du magasin de paquets et de la console
        cx.subscribe(&store, |this, _emitter, event, cx| match event {
            PackageStoreEvent::ResultsChanged => {
                cx.notify();
            }
            PackageStoreEvent::PackageInvalidated(key) => {
                if this.session.read(cx).selected_package_key.as_ref() == Some(key) {
                    this.ensure_package_details(key.clone(), cx);
                }
                cx.notify();
            }
            PackageStoreEvent::UpdatesChanged(_) => {
                cx.notify();
            }
            PackageStoreEvent::InstalledChanged(_) => {
                cx.notify();
            }
        })
        .detach();

        cx.subscribe(&console, |this, _emitter, event, cx| {
            if let ConsoleEvent::OperationFinished(status) = event {
                let reduce = this.gpui_config.reduce_motion;
                let (kind, title, msg, action) = match status {
                    crate::state::OperationStatus::Success(m) => {
                        (ToastKind::Success, "Operation successful", m.clone(), None)
                    }
                    crate::state::OperationStatus::Error(m) => (
                        ToastKind::Error,
                        "Operation failed",
                        m.clone(),
                        Some(ToastAction::OpenLogs),
                    ),
                    _ => return,
                };
                this.toast_center.update(cx, |tc, cx| {
                    tc.post(kind, title, msg, action, reduce, cx);
                });
                cx.notify();
            }
        })
        .detach();

        let view = Self {
            session,
            store,
            console,
            toast_center,
            workstation,
            sidebar,
            console_view,
            settings,
            shelly_settings,
            gpui_config: gpui_config.clone(),
            theme,
            search_focus: cx.focus_handle(),
            search_input_buffer: String::new(),
            search_debounce_task: None,
            news: Vec::new(),
            is_loading_news: false,
            motion_policy: crate::state::motion::MotionPolicy::new(gpui_config.reduce_motion),
        };

        // Liaison des mutations depuis PackageWorkstationView vers WorkspaceView
        let entity_ws = cx.entity().clone();
        view.workstation.update(cx, |ws, _cx| {
            let entity_mut = entity_ws.clone();
            ws.set_on_mutation(Rc::new(move |pkg, is_install, _window, cx| {
                let pkg_clone = pkg.clone();
                let key = pkg.key();
                let is_aur = pkg.source_type == "AUR";
                let is_flatpak = pkg.source_type == "Flatpak";
                entity_mut.update(cx, |view, cx| {
                    if is_install {
                        view.run_package_mutation(
                            &format!("Installation of {}", pkg_clone.name),
                            MutationAction::Install { is_aur, is_flatpak },
                            pkg_clone.name.clone(),
                            key,
                            cx,
                        );
                    } else {
                        view.run_package_mutation(
                            &format!("Uninstallation of {}", pkg_clone.name),
                            MutationAction::Remove { is_flatpak },
                            pkg_clone.name.clone(),
                            key,
                            cx,
                        );
                    }
                });
            }));
            let entity_upg = entity_ws.clone();
            ws.set_on_upgrade_all(Rc::new(move |_window, cx| {
                entity_upg.update(cx, |view, cx| {
                    view.run_package_mutation(
                        "Full system upgrade",
                        MutationAction::UpgradeSystem,
                        "system".to_string(),
                        PackageKey::new(PackageSourceKind::Alpm, "system-upgrade", None),
                        cx,
                    );
                });
            }));
            let entity_retry = entity_ws.clone();
            ws.on_retry_source = Some(Rc::new(move |kind, _w, cx| {
                entity_retry.update(cx, |view, cx| {
                    view.retry_source_search(kind, cx);
                });
            }));
            let entity_rel_inst = entity_ws.clone();
            ws.on_reload_installed = Some(Rc::new(move |_w, cx| {
                entity_rel_inst.update(cx, |view, cx| {
                    view.load_installed_packages(cx);
                });
            }));
            let entity_rel_upd = entity_ws.clone();
            ws.on_reload_updates = Some(Rc::new(move |_w, cx| {
                entity_rel_upd.update(cx, |view, cx| {
                    view.load_updates(cx);
                });
            }));
            let entity_rel_det = entity_ws.clone();
            ws.on_retry_details = Some(Rc::new(move |key, _w, cx| {
                entity_rel_det.update(cx, |view, cx| {
                    view.ensure_package_details(key.clone(), cx);
                });
            }));
        });

        // Chargement initial asynchrone non-bloquant
        view.trigger_initial_load(cx);

        view
    }

    /// Déclenche le chargement initial en arrière-plan
    fn trigger_initial_load(&self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            let res = client.list_updates().await;
            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(updates) => {
                        let unified: Vec<UnifiedPackage> = updates
                            .into_iter()
                            .map(UnifiedPackage::from_update)
                            .collect();
                        st.updates_error = None;
                        st.set_updates_packages(unified, cx);
                    }
                    Err(e) => {
                        log::warn!("Initial updates check failed: {:?}", e);
                        st.updates_error = Some(e.to_string());
                    }
                });
            });
        })
        .detach();

        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            let res = client.search_installed("").await;
            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(installed) => {
                        let unified: Vec<UnifiedPackage> = installed
                            .into_iter()
                            .map(|p| UnifiedPackage::from_alpm(p, true))
                            .collect();
                        st.installed_error = None;
                        st.set_installed_packages(unified, cx);
                    }
                    Err(e) => {
                        log::warn!("Initial installed packages check failed: {:?}", e);
                        st.installed_error = Some(e.to_string());
                    }
                });
            });
        })
        .detach();
    }

    /// Réaction au changement de destination
    fn on_destination_changed(&mut self, dest: NavDestination, cx: &mut Context<Self>) {
        self.workstation.update(cx, |ws, _cx| {
            ws.scroll_handle.scroll_to_item(0, ScrollStrategy::Top)
        });

        match dest {
            NavDestination::Browse => {
                let query = self.session.read(cx).search_query.clone();
                if !query.trim().is_empty() {
                    self.execute_search(query, cx);
                }
            }
            NavDestination::Installed => {
                let is_empty = self.store.read(cx).installed_packages.is_empty();
                if is_empty {
                    self.load_installed_packages(cx);
                }
            }
            NavDestination::Updates => {
                let is_empty = self.store.read(cx).updates_packages.is_empty();
                if is_empty {
                    self.load_updates(cx);
                }
            }
            NavDestination::News => {
                if self.news.is_empty() {
                    self.load_news(cx);
                }
            }
            NavDestination::Settings => {}
        }
    }

    /// Charge les paquets installés localement via le backend Shelly
    fn load_installed_packages(&mut self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            let res = client.search_installed("").await;
            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(packages) => {
                        let unified: Vec<UnifiedPackage> = packages
                            .into_iter()
                            .map(|p| UnifiedPackage::from_alpm(p, true))
                            .collect();
                        st.installed_error = None;
                        st.set_installed_packages(unified, cx);
                    }
                    Err(e) => {
                        log::error!("Failed to load installed packages: {:?}", e);
                        st.installed_error = Some(e.to_string());
                        st.set_installed_packages(Vec::new(), cx);
                    }
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Charge les mises à jour disponibles via le backend Shelly
    fn load_updates(&mut self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            let res = client.list_updates().await;
            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(packages) => {
                        let unified: Vec<UnifiedPackage> = packages
                            .into_iter()
                            .map(UnifiedPackage::from_update)
                            .collect();
                        st.updates_error = None;
                        st.set_updates_packages(unified, cx);
                    }
                    Err(e) => {
                        log::error!("Failed to list updates: {:?}", e);
                        st.updates_error = Some(e.to_string());
                        st.set_updates_packages(Vec::new(), cx);
                    }
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Charge les actualités officielles Arch Linux
    fn load_news(&mut self, cx: &mut Context<Self>) {
        self.is_loading_news = true;
        let client = self.store.read(cx).client.clone();

        cx.spawn(async move |this, cx| {
            let res = client.list_news().await;
            let _ = this.update(cx, |view, cx| {
                view.is_loading_news = false;
                match res {
                    Ok(news_items) => {
                        view.news = news_items;
                        view.store.update(cx, |st, _cx| {
                            st.news_error = None;
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to list news: {:?}", e);
                        view.news = Vec::new();
                        view.store.update(cx, |st, _cx| {
                            st.news_error = Some(e.to_string());
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// S'assure que la fiche détaillée du paquet est disponible dans le cache
    fn ensure_package_details(&mut self, key: PackageKey, cx: &mut Context<Self>) {
        if self.store.read(cx).get_cached_details(&key).is_some() {
            return;
        }

        if key.source == PackageSourceKind::Alpm {
            let client = self.store.read(cx).client.clone();
            let pkg_name = key.name.clone();
            let key_clone = key.clone();

            cx.spawn(async move |this, cx| {
                let res = client.get_package_details(&pkg_name).await;
                let _ = this.update(cx, |view, cx| {
                    view.store.update(cx, |st, _cx| match res {
                        Ok(Some(details)) => {
                            st.cache_details(key_clone, details);
                        }
                        Ok(None) => {
                            st.set_detail_error(key_clone, "Package details not found".to_string());
                        }
                        Err(e) => {
                            st.set_detail_error(key_clone, e.to_string());
                        }
                    });
                    cx.notify();
                });
            })
            .detach();
        }
    }

    /// S'assure que la recette PKGBUILD d'un paquet AUR est disponible dans le cache
    fn ensure_pkgbuild(&mut self, pkg_name: String, cx: &mut Context<Self>) {
        if self.store.read(cx).get_cached_pkgbuild(&pkg_name).is_some() {
            return;
        }

        let client = self.store.read(cx).client.clone();
        let name_clone = pkg_name.clone();

        self.workstation.update(cx, |ws, cx| {
            ws.is_loading_pkgbuild = true;
            cx.notify();
        });

        cx.spawn(async move |this, cx| {
            let res = client.fetch_aur_pkgbuild(&name_clone).await;
            let _ = this.update(cx, |view, cx| {
                view.workstation.update(cx, |ws, cx| {
                    ws.is_loading_pkgbuild = false;
                    cx.notify();
                });
                match res {
                    Ok(content) => {
                        view.store.update(cx, |st, _cx| {
                            st.cache_pkgbuild(name_clone, content);
                        });
                    }
                    Err(err) => {
                        log::warn!(
                            "Échec de récupération du PKGBUILD pour {}: {:?}",
                            name_clone,
                            err
                        );
                        let reduce = view.gpui_config.reduce_motion;
                        view.toast_center.update(cx, |tc, cx| {
                            tc.post(
                                ToastKind::Warning,
                                "PKGBUILD unavailable",
                                format!("Failed to load PKGBUILD for {}", name_clone),
                                None,
                                reduce,
                                cx,
                            );
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Déclenche la recherche avec temporisation (debounce)
    fn on_search_input(&mut self, input: String, cx: &mut Context<Self>) {
        self.search_input_buffer = input.clone();

        if input.trim().is_empty() {
            self.search_debounce_task = None;
            self.execute_search(String::new(), cx);
            return;
        }

        self.search_debounce_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(60))
                .await;
            let _ = this.update(cx, |view, cx| {
                view.execute_search(input, cx);
            });
        }));
    }

    /// Réessaye la recherche pour une source spécifique après échec
    fn retry_source_search(&mut self, kind: PackageSourceKind, cx: &mut Context<Self>) {
        let query = self.session.read(cx).search_query.trim().to_string();
        if query.is_empty() {
            return;
        }

        let aur_enabled = self.shelly_settings.aur_enabled;
        let flatpak_enabled = self.shelly_settings.flat_pack_enabled;
        let appimage_enabled = self.shelly_settings.app_image_enabled;
        let client = self.store.read(cx).client.clone();
        let query_clone = query.clone();

        cx.spawn(async move |this, cx| {
            let res: Result<Vec<UnifiedPackage>, String> = match kind {
                PackageSourceKind::Alpm => client
                    .search_standard(&query_clone)
                    .await
                    .map(|pkgs| {
                        pkgs.into_iter()
                            .map(|p| UnifiedPackage::from_alpm(p, false))
                            .collect()
                    })
                    .map_err(|e| e.to_string()),
                PackageSourceKind::Aur => {
                    if aur_enabled {
                        client
                            .search_aur(&query_clone)
                            .await
                            .map(|pkgs| {
                                pkgs.into_iter()
                                    .map(|p| UnifiedPackage::from_aur(p, false))
                                    .collect()
                            })
                            .map_err(|e| e.to_string())
                    } else {
                        Ok(Vec::new())
                    }
                }
                PackageSourceKind::Flatpak => {
                    if flatpak_enabled {
                        client
                            .search_flatpak(&query_clone)
                            .await
                            .map(|pkgs| {
                                pkgs.into_iter()
                                    .map(|p| UnifiedPackage::from_flatpak(p, false))
                                    .collect()
                            })
                            .map_err(|e| e.to_string())
                    } else {
                        Ok(Vec::new())
                    }
                }
                PackageSourceKind::AppImage => {
                    if appimage_enabled {
                        client
                            .list_appimages()
                            .await
                            .map(|appimages| {
                                let q_lower = query_clone.to_lowercase();
                                appimages
                                    .into_iter()
                                    .filter(|ai| {
                                        ai.name.to_lowercase().contains(&q_lower)
                                            || ai
                                                .desktop_name
                                                .as_ref()
                                                .map(|d| d.to_lowercase().contains(&q_lower))
                                                .unwrap_or(false)
                                            || ai
                                                .description
                                                .as_ref()
                                                .map(|d| d.to_lowercase().contains(&q_lower))
                                                .unwrap_or(false)
                                    })
                                    .map(UnifiedPackage::from_appimage)
                                    .collect()
                            })
                            .map_err(|e| e.to_string())
                    } else {
                        Ok(Vec::new())
                    }
                }
            };

            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(new_pkgs) => {
                        st.update_source_health(kind, SourceHealth::ready());
                        let mut current: Vec<UnifiedPackage> = st
                            .active_results
                            .iter()
                            .filter(|p| p.key().source != kind)
                            .cloned()
                            .collect();
                        current.extend(new_pkgs);
                        let gen = view.session.read(cx).search_generation;
                        st.set_active_results(current, gen, cx);

                        let scope = view.session.read(cx).source_scope;
                        let all_healthy = scope
                            .enabled_sources(aur_enabled, flatpak_enabled, appimage_enabled)
                            .iter()
                            .all(|src| {
                                st.source_health
                                    .get(src)
                                    .map(|h| !h.is_failed())
                                    .unwrap_or(true)
                            });
                        if all_healthy {
                            st.cache_search(&query_clone, scope, st.active_results.to_vec());
                        }
                    }
                    Err(err) => {
                        st.update_source_health(kind, SourceHealth::failed(err));
                    }
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Exécute la recherche unifiée multi-sources avec cache de session et protection contre les requêtes obsolètes
    fn execute_search(&mut self, query: String, cx: &mut Context<Self>) {
        let trimmed = query.trim().to_string();

        if trimmed.is_empty() {
            let next_gen = self.session.update(cx, |s, cx| {
                s.search_query.clear();
                s.set_searching(false, cx);
                s.select_package(None, cx);
                s.next_search_generation()
            });
            self.store.update(cx, |st, cx| {
                st.set_active_results(Vec::new(), next_gen, cx);
            });
            return;
        }

        let source_scope = self.session.read(cx).source_scope;

        // 1. Vérification du cache de session
        if let Some(cached) = self
            .store
            .read(cx)
            .get_cached_search(&trimmed, source_scope)
            .cloned()
        {
            let gen = self.session.update(cx, |s, _cx| {
                s.search_query = trimmed.clone();
                s.is_searching = false;
                s.next_search_generation()
            });
            self.store.update(cx, |st, cx| {
                st.set_active_results(cached, gen, cx);
            });
            self.workstation.update(cx, |ws, _cx| {
                ws.scroll_handle.scroll_to_item(0, ScrollStrategy::Top)
            });
            return;
        }

        // 2. Requête backend concurrente
        let gen = self.session.update(cx, |s, _cx| {
            s.search_query = trimmed.clone();
            s.is_searching = true;
            s.next_search_generation()
        });

        let aur_enabled = self.shelly_settings.aur_enabled;
        let flatpak_enabled = self.shelly_settings.flat_pack_enabled;
        let appimage_enabled = self.shelly_settings.app_image_enabled;

        let client = self.store.read(cx).client.clone();
        let query_clone = trimmed.clone();

        cx.spawn(async move |this, cx| {
            let (alpm_res, aur_res, flatpak_res, appimage_res) = tokio::join!(
                async {
                    if source_scope.alpm {
                        Some(client.search_standard(&query_clone).await)
                    } else {
                        None
                    }
                },
                async {
                    if source_scope.aur && aur_enabled {
                        Some(client.search_aur(&query_clone).await)
                    } else {
                        None
                    }
                },
                async {
                    if source_scope.flatpak && flatpak_enabled {
                        Some(client.search_flatpak(&query_clone).await)
                    } else {
                        None
                    }
                },
                async {
                    if source_scope.appimage && appimage_enabled {
                        Some(client.list_appimages().await)
                    } else {
                        None
                    }
                }
            );

            let _ = this.update(cx, |view, cx| {
                let current_gen = view.session.read(cx).search_generation;
                if current_gen != gen {
                    return;
                }

                let mut combined = Vec::new();
                let mut all_succeeded = true;

                view.store.update(cx, |st, cx| {
                    if let Some(res) = alpm_res {
                        match res {
                            Ok(pkgs) => {
                                st.update_source_health(
                                    PackageSourceKind::Alpm,
                                    SourceHealth::ready(),
                                );
                                for p in pkgs {
                                    combined.push(UnifiedPackage::from_alpm(p, false));
                                }
                            }
                            Err(e) => {
                                all_succeeded = false;
                                st.update_source_health(
                                    PackageSourceKind::Alpm,
                                    SourceHealth::failed(e.to_string()),
                                );
                            }
                        }
                    }

                    if let Some(res) = aur_res {
                        match res {
                            Ok(pkgs) => {
                                st.update_source_health(
                                    PackageSourceKind::Aur,
                                    SourceHealth::ready(),
                                );
                                for p in pkgs {
                                    combined.push(UnifiedPackage::from_aur(p, false));
                                }
                            }
                            Err(e) => {
                                all_succeeded = false;
                                st.update_source_health(
                                    PackageSourceKind::Aur,
                                    SourceHealth::failed(e.to_string()),
                                );
                            }
                        }
                    }

                    if let Some(res) = flatpak_res {
                        match res {
                            Ok(pkgs) => {
                                st.update_source_health(
                                    PackageSourceKind::Flatpak,
                                    SourceHealth::ready(),
                                );
                                for p in pkgs {
                                    combined.push(UnifiedPackage::from_flatpak(p, false));
                                }
                            }
                            Err(e) => {
                                all_succeeded = false;
                                st.update_source_health(
                                    PackageSourceKind::Flatpak,
                                    SourceHealth::failed(e.to_string()),
                                );
                            }
                        }
                    }

                    if let Some(res) = appimage_res {
                        match res {
                            Ok(appimages) => {
                                st.update_source_health(
                                    PackageSourceKind::AppImage,
                                    SourceHealth::ready(),
                                );
                                let q_lower = query_clone.to_lowercase();
                                for ai in appimages {
                                    let matches = ai.name.to_lowercase().contains(&q_lower)
                                        || ai
                                            .desktop_name
                                            .as_ref()
                                            .map(|d| d.to_lowercase().contains(&q_lower))
                                            .unwrap_or(false)
                                        || ai
                                            .description
                                            .as_ref()
                                            .map(|d| d.to_lowercase().contains(&q_lower))
                                            .unwrap_or(false);
                                    if matches {
                                        combined.push(UnifiedPackage::from_appimage(ai));
                                    }
                                }
                            }
                            Err(e) => {
                                all_succeeded = false;
                                st.update_source_health(
                                    PackageSourceKind::AppImage,
                                    SourceHealth::failed(e.to_string()),
                                );
                            }
                        }
                    }

                    if all_succeeded {
                        st.cache_search(&query_clone, source_scope, combined.clone());
                    }

                    st.set_active_results(combined, gen, cx);
                });

                view.session.update(cx, |s, cx| {
                    s.set_searching(false, cx);
                });
                view.workstation.update(cx, |ws, _cx| {
                    ws.scroll_handle.scroll_to_item(0, ScrollStrategy::Top)
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Exécute une opération de mutation (installation, suppression, mise à jour) avec streaming
    fn run_package_mutation(
        &mut self,
        action_name: &str,
        action: MutationAction,
        name: String,
        key: PackageKey,
        cx: &mut Context<Self>,
    ) {
        if self.console.read(cx).is_running() {
            return;
        }

        self.console.update(cx, |c, cx| {
            c.start_operation(action_name, cx);
        });

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        let client = self.store.read(cx).client.clone();

        match action {
            MutationAction::Install { is_aur, is_flatpak } => {
                client.install_package(&name, is_aur, is_flatpak, tx);
            }
            MutationAction::Remove { is_flatpak } => {
                let cascade = self.shelly_settings.package_management_cascade_delete;
                let remove_configs = self.shelly_settings.package_management_remove_configs;
                client.remove_package(&name, is_flatpak, cascade, remove_configs, tx);
            }
            MutationAction::UpgradeSystem => {
                client.upgrade_system(tx);
            }
        }

        cx.spawn(async move |this, cx| {
            let mut final_status = false;
            let mut final_msg = String::from("Operation completed");

            while let Some(event) = rx.recv().await {
                match event {
                    LogStreamEvent::Line(line) => {
                        let _ = this.update(cx, |view, cx| {
                            view.console.update(cx, |c, cx| c.append_stdout(&line, cx));
                        });
                    }
                    LogStreamEvent::ErrorLine(line) => {
                        let _ = this.update(cx, |view, cx| {
                            view.console.update(cx, |c, cx| c.append_stderr(&line, cx));
                        });
                    }
                    LogStreamEvent::Finished(success, code) => {
                        final_status = success;
                        final_msg = if success {
                            format!("Success (exit code {:?})", code)
                        } else {
                            format!("Operation failed (exit code {:?})", code)
                        };
                    }
                }
            }

            let _ = this.update(cx, |view, cx| {
                view.console.update(cx, |c, cx| {
                    c.finish_operation(final_status, &final_msg, cx);
                });

                if final_status {
                    view.store.update(cx, |st, cx| {
                        st.invalidate_package(&key, cx);
                        st.invalidate_installed(cx);
                        st.invalidate_updates(cx);
                    });

                    let dest = view.session.read(cx).destination;
                    match dest {
                        NavDestination::Installed => view.load_installed_packages(cx),
                        NavDestination::Updates => view.load_updates(cx),
                        _ => {}
                    }
                }
            });
        })
        .detach();
    }

    /// Sauvegarde les paramètres modifiés et met à jour l'application
    pub fn save_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let bounds = window.window_bounds();
        if let WindowBounds::Windowed(b) = bounds {
            let raw_w = b.size.width / px(1.0);
            let raw_h = b.size.height / px(1.0);
            let (w, h) = ConfigManager::sanitize_window_size(raw_w, raw_h);
            self.settings.draft_gpui.window_width = w;
            self.settings.draft_gpui.window_height = h;
        }

        self.settings.draft_gpui.last_selected_tab = self
            .session
            .read(cx)
            .last_workspace_destination
            .workspace_config_index()
            .unwrap_or(0);

        let draft_aur = self.settings.draft_shelly.aur_enabled;
        let draft_flatpak = self.settings.draft_shelly.flat_pack_enabled;
        let draft_appimage = self.settings.draft_shelly.app_image_enabled;

        let mut scope = self.session.read(cx).source_scope;
        let mut scope_changed = false;
        if !draft_aur && scope.aur {
            scope.aur = false;
            scope_changed = true;
        }
        if !draft_flatpak && scope.flatpak {
            scope.flatpak = false;
            scope_changed = true;
        }
        if !draft_appimage && scope.appimage {
            scope.appimage = false;
            scope_changed = true;
        }

        if scope_changed {
            self.session.update(cx, |s, cx| {
                s.set_source_scope(scope, cx);
            });
        }

        match self.settings.save() {
            Ok(()) => {
                let sources_changed = self.shelly_settings.aur_enabled != draft_aur
                    || self.shelly_settings.flat_pack_enabled != draft_flatpak
                    || self.shelly_settings.app_image_enabled != draft_appimage;

                self.shelly_settings = self.settings.draft_shelly.clone();
                self.gpui_config = self.settings.draft_gpui.clone();
                self.theme = if self.gpui_config.dark_theme {
                    Theme::dark()
                } else {
                    Theme::light()
                };

                let theme = self.theme;
                self.motion_policy =
                    crate::state::motion::MotionPolicy::new(self.gpui_config.reduce_motion);
                let reduce = self.motion_policy.reduce_motion;

                self.workstation.update(cx, |ws, cx| {
                    ws.set_theme(theme, cx);
                    ws.set_reduce_motion(reduce, cx);
                    ws.set_compact(self.gpui_config.compact_view, cx);
                    ws.set_sources_enabled(draft_aur, draft_flatpak, draft_appimage, cx);
                });

                if sources_changed {
                    self.session.update(cx, |s, cx| {
                        s.clamp_source_scope(draft_aur, draft_flatpak, draft_appimage, cx);
                    });
                }

                self.sidebar.update(cx, |sb, cx| {
                    sb.set_theme(theme, cx);
                    sb.set_reduce_motion(reduce, cx);
                });

                self.console_view.update(cx, |cv, cx| {
                    cv.set_theme(theme, cx);
                    cv.set_reduce_motion(reduce, cx);
                    cv.set_configured_height(self.gpui_config.log_drawer_height, cx);
                });

                self.console.update(cx, |c, _| {
                    c.set_auto_open(self.gpui_config.log_drawer_open);
                });

                if sources_changed {
                    self.store.update(cx, |st, _| {
                        st.search_cache.clear();
                    });
                    let query = self.session.read(cx).search_query.clone();
                    if !query.trim().is_empty() {
                        self.execute_search(query, cx);
                    }
                }

                self.toast_center.update(cx, |tc, cx| {
                    tc.post(
                        ToastKind::Success,
                        "Settings saved",
                        "Your preferences have been saved to disk.",
                        None,
                        reduce,
                        cx,
                    );
                });
                cx.notify();
            }
            Err(e) => {
                let reduce = self.gpui_config.reduce_motion;
                self.toast_center.update(cx, |tc, cx| {
                    tc.post(
                        ToastKind::Error,
                        "Failed to save settings",
                        format!("{}", e),
                        Some(ToastAction::OpenLogs),
                        reduce,
                        cx,
                    );
                });
                cx.notify();
            }
        }
    }

    /// Récupère la liste de paquets actuellement affichée selon la destination active
    fn current_display_packages<'a>(&self, cx: &'a Context<Self>) -> &'a [UnifiedPackage] {
        let dest = self.session.read(cx).destination;
        let store = self.store.read(cx);
        match dest {
            NavDestination::Browse => &store.active_results,
            NavDestination::Installed => &store.installed_packages,
            NavDestination::Updates => &store.updates_packages,
            _ => &[],
        }
    }

    /// Navigation au clavier dans la liste virtuelle (flèches Haut/Bas)
    pub fn on_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        if key == "tab" {
            if event.keystroke.modifiers.shift {
                window.focus_prev();
            } else {
                window.focus_next();
            }
            return;
        }

        if key == "/" && !event.keystroke.modifiers.control && !event.keystroke.modifiers.alt {
            let search_input = self.workstation.read(cx).search_input.clone();
            let is_focused = search_input.read(cx).is_focused(window);
            if !is_focused {
                search_input.update(cx, |si, _cx| si.focus(window));
                return;
            }
        }

        let packages = self.current_display_packages(cx);
        if packages.is_empty() {
            return;
        }

        let selected_key = self.session.read(cx).selected_package_key.clone();

        let current_idx = selected_key
            .as_ref()
            .and_then(|sk| packages.iter().position(|p| &p.key() == sk));

        match key {
            "down" => {
                let next_idx = match current_idx {
                    Some(idx) => (idx + 1).min(packages.len().saturating_sub(1)),
                    None => 0,
                };
                let next_pkg = &packages[next_idx];
                let next_key = next_pkg.key();
                self.session.update(cx, |s, cx| {
                    s.select_package(Some(next_key), cx);
                });
                self.workstation.update(cx, |ws, _cx| {
                    ws.scroll_handle
                        .scroll_to_item_strict(next_idx, ScrollStrategy::Top)
                });
            }
            "up" => {
                let prev_idx = match current_idx {
                    Some(idx) => idx.saturating_sub(1),
                    None => 0,
                };
                let prev_pkg = &packages[prev_idx];
                let prev_key = prev_pkg.key();
                self.session.update(cx, |s, cx| {
                    s.select_package(Some(prev_key), cx);
                });
                self.workstation.update(cx, |ws, _cx| {
                    ws.scroll_handle
                        .scroll_to_item_strict(prev_idx, ScrollStrategy::Top)
                });
            }
            "escape" => {
                self.session.update(cx, |s, cx| {
                    s.select_package(None, cx);
                });
            }
            _ => {}
        }
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let entity = cx.entity().clone();

        // ── Lecture de l'état de session ────────────────────────────────────
        let (destination, destination_epoch) = {
            let session = self.session.read(cx);
            (session.destination, session.destination_epoch)
        };

        // ── 1. Contenu principal selon la destination ───────────────────────
        let main_content = match destination {
            NavDestination::Browse | NavDestination::Installed | NavDestination::Updates => {
                div().size_full().child(self.workstation.clone())
            }
            NavDestination::News => {
                let entity_retry = entity.clone();
                let store = self.store.read(cx);
                div().size_full().child(NewsView::render(NewsViewProps {
                    news: &self.news,
                    error: store.news_error.as_deref(),
                    is_loading: self.is_loading_news,
                    theme: &theme,
                    on_retry: Some(Rc::new(move |_w, cx| {
                        entity_retry.update(cx, |view, cx| {
                            view.load_news(cx);
                        });
                    })),
                }))
            }
            NavDestination::Settings => {
                let entity_st = entity.clone();
                let theme_ref = &theme;
                div()
                    .size_full()
                    .child(SettingsView::render(SettingsViewProps {
                        shelly_settings: &self.settings.draft_shelly,
                        gpui_config: &self.settings.draft_gpui,
                        is_dirty: self.settings.is_dirty,
                        theme: theme_ref,
                        on_toggle_aur: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_aur();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_flatpak: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_flatpak();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_appimage: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_appimage();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_cascade_delete: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_cascade_delete();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_remove_configs: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_remove_configs();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_dark_theme: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_dark_theme();
                                    let theme = if view.settings.draft_gpui.dark_theme {
                                        Theme::dark()
                                    } else {
                                        Theme::light()
                                    };
                                    view.theme = theme;
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_theme(theme, cx));
                                    view.sidebar.update(cx, |sb, cx| sb.set_theme(theme, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_theme(theme, cx));
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_compact_view: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_compact_view();
                                    let compact = view.settings.draft_gpui.compact_view;
                                    view.session
                                        .update(cx, |s, cx| s.set_sidebar_collapsed(compact, cx));
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_compact(compact, cx));
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_log_drawer_auto_open: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_log_drawer_auto_open();
                                    let auto_open = view.settings.draft_gpui.log_drawer_open;
                                    view.console.update(cx, |c, _| c.set_auto_open(auto_open));
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_reduce_motion: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_reduce_motion();
                                    let live_reduce = view.settings.draft_gpui.reduce_motion;
                                    view.gpui_config.reduce_motion = live_reduce;
                                    view.motion_policy =
                                        crate::state::motion::MotionPolicy::new(live_reduce);
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_reduce_motion(live_reduce, cx));
                                    view.sidebar
                                        .update(cx, |sb, cx| sb.set_reduce_motion(live_reduce, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_reduce_motion(live_reduce, cx));
                                    cx.notify();
                                })
                            })
                        },
                        on_save: {
                            let e = entity_st.clone();
                            Rc::new(move |w, cx| {
                                e.update(cx, |view, cx| {
                                    view.save_settings(w, cx);
                                })
                            })
                        },
                        on_reset: Some({
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.reset_to(
                                        view.shelly_settings.clone(),
                                        view.gpui_config.clone(),
                                    );
                                    let theme = if view.gpui_config.dark_theme {
                                        Theme::dark()
                                    } else {
                                        Theme::light()
                                    };
                                    view.theme = theme;
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_theme(theme, cx));
                                    view.sidebar.update(cx, |sb, cx| sb.set_theme(theme, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_theme(theme, cx));
                                    let compact = view.gpui_config.compact_view;
                                    view.session
                                        .update(cx, |s, cx| s.set_sidebar_collapsed(compact, cx));
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_compact(compact, cx));
                                    let auto_open = view.gpui_config.log_drawer_open;
                                    view.console.update(cx, |c, _| c.set_auto_open(auto_open));
                                    let reduce = view.gpui_config.reduce_motion;
                                    view.motion_policy =
                                        crate::state::motion::MotionPolicy::new(reduce);
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_reduce_motion(reduce, cx));
                                    view.sidebar
                                        .update(cx, |sb, cx| sb.set_reduce_motion(reduce, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_reduce_motion(reduce, cx));
                                    cx.notify();
                                })
                            })
                        }),
                    }))
            }
        };

        // Transition discrète de destination (120ms d'opacité ou immédiat sous reduce_motion)
        let reduce_motion = self.motion_policy.reduce_motion;
        let destination_element = if reduce_motion {
            div()
                .id("dest_view")
                .size_full()
                .child(main_content)
                .into_any_element()
        } else {
            div()
                .id("dest_view")
                .size_full()
                .child(main_content)
                .with_animation(
                    ("dest-fade", destination_epoch as usize),
                    Animation::new(MotionDurations::FAST).with_easing(gpui::ease_out_quint()),
                    |elem, delta| elem.opacity(delta),
                )
                .into_any_element()
        };

        // ── 2. Overlay de notifications (Toasts) ────────────────────────────
        let toasts = self.toast_center.read(cx).toasts.clone();
        let entity_toast_dismiss = entity.clone();
        let entity_toast_action = entity.clone();

        let toast_overlay = ToastOverlay::render(ToastOverlayProps {
            toasts: &toasts,
            reduce_motion,
            theme: &theme,
            on_dismiss: Rc::new(move |id, _w, cx| {
                entity_toast_dismiss.update(cx, |view, cx| {
                    let reduce = view.gpui_config.reduce_motion;
                    view.toast_center.update(cx, |tc, cx| {
                        tc.dismiss(id, reduce, cx);
                    });
                });
            }),
            on_action: Rc::new(move |action, _w, cx| {
                entity_toast_action.update(cx, |view, cx| match action {
                    ToastAction::OpenLogs => {
                        view.console.update(cx, |c, cx| c.set_open(true, cx));
                    }
                });
            }),
        });

        // ── 3. Assemblage final du shell ─────────────────────────────────────
        let entity_key = entity.clone();

        div()
            .id("workspace_root")
            .size_full()
            .relative()
            .flex()
            .flex_row()
            .bg(theme.bg_app)
            .text_color(theme.text_primary)
            .track_focus(&self.search_focus)
            .on_key_down(move |event, window, cx| {
                entity_key.update(cx, |view, cx| {
                    view.on_key_down(event, window, cx);
                });
            })
            .child(self.sidebar.clone())
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(destination_element),
                    )
                    .child(self.console_view.clone()),
            )
            .child(toast_overlay)
    }
}
