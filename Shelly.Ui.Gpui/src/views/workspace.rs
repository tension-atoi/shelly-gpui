use crate::backend::client::ShellyClient;
use crate::backend::models::{ArchNewsItem, UnifiedPackage};
use crate::backend::process::LogStreamEvent;
use crate::components::log_drawer::{LogDrawer, LogDrawerProps};
use crate::components::sidebar::{Sidebar, SidebarProps};
use crate::components::toast_overlay::{ToastOverlay, ToastOverlayProps};
use crate::config::{ConfigManager, GpuiUiConfig, ShellySettings};
use crate::state::{
    AnimatedScalar, AppSession, ConsoleEvent, ConsoleModel, InspectorTab, MotionDurations,
    NavDestination, PackageKey, PackageSourceKind, PackageStore, PackageStoreEvent, SessionEvent,
    SourceFilter, ToastAction, ToastCenter, ToastKind,
};
use crate::theme::Theme;
use crate::views::news::{NewsView, NewsViewProps};
use crate::views::package_workstation::PackageWorkstationView;
use crate::views::settings::{SettingsView, SettingsViewProps};
use gpui::ScrollStrategy;
use gpui::*;
use std::rc::Rc;
use std::time::Instant;
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
    pub settings: SettingsView,
    pub shelly_settings: ShellySettings,
    pub gpui_config: GpuiUiConfig,
    pub theme: Theme,
    pub search_focus: FocusHandle,
    pub search_input_buffer: String,
    pub search_debounce_task: Option<Task<()>>,
    pub news: Vec<ArchNewsItem>,
    pub is_loading_news: bool,
    pub is_mutating: bool,
    pub sidebar_width_scalar: AnimatedScalar,
    pub console_height_scalar: AnimatedScalar,
    pub motion_policy: crate::state::motion::MotionPolicy,
    pub logs_copied_feedback: bool,
}

impl WorkspaceView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let shelly_settings = ConfigManager::load_shelly_settings();
        let gpui_config = ConfigManager::load_gpui_config();
        let theme = if gpui_config.dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        };

        let client = ShellyClient::new(None);
        let session = cx.new(|_cx| {
            let mut s = AppSession::new();
            if gpui_config.compact_view {
                s.sidebar_collapsed = true;
            }
            s.destination = match gpui_config.last_selected_tab {
                1 => NavDestination::Installed,
                2 => NavDestination::Updates,
                3 => NavDestination::News,
                4 => NavDestination::Settings,
                _ => NavDestination::Browse,
            };
            s
        });
        let store = cx.new(|_cx| PackageStore::new(client));
        let console = cx.new(|_cx| {
            let mut c = ConsoleModel::new();
            c.is_open = gpui_config.log_drawer_open;
            c
        });
        let toast_center = cx.new(|_cx| ToastCenter::new());

        let reduce_motion = gpui_config.reduce_motion;
        let workstation = cx.new(|cx| {
            PackageWorkstationView::new(
                session.clone(),
                store.clone(),
                toast_center.clone(),
                theme,
                reduce_motion,
                cx,
            )
        });

        let sidebar_width_scalar = AnimatedScalar::new(if gpui_config.compact_view {
            56.0
        } else {
            190.0
        });

        let mut console_height_scalar = AnimatedScalar::new(if gpui_config.log_drawer_open {
            gpui_config.log_drawer_height
        } else {
            0.0
        });
        console_height_scalar.easing = crate::state::motion::ease_in_out;

        let settings = SettingsView::new(shelly_settings.clone(), gpui_config.clone());

        // Abonnements réactifs aux événements de session
        cx.subscribe(&session, |this, _emitter, event, cx| match event {
            SessionEvent::DestinationChanged(dest) => {
                log::debug!("Navigation vers la destination : {:?}", dest);
                this.on_destination_changed(*dest, cx);
                cx.notify();
            }
            SessionEvent::SourceFilterChanged(filter) => {
                log::debug!("Changement de filtre de source : {:?}", filter);
                let query = this.session.read(cx).search_query.clone();
                if !query.trim().is_empty() {
                    this.execute_search(query, cx);
                }
                cx.notify();
            }
            SessionEvent::SearchQueryChanged(q) => {
                log::debug!("Requête de recherche mise à jour : '{}'", q);
                this.on_search_input(q.clone(), cx);
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
            SessionEvent::SidebarToggled(collapsed) => {
                let target = if *collapsed { 56.0 } else { 190.0 };
                this.sidebar_width_scalar.retarget(
                    target,
                    MotionDurations::STANDARD,
                    Instant::now(),
                    this.gpui_config.reduce_motion,
                );
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

        cx.subscribe(&console, |this, _emitter, event, cx| match event {
            ConsoleEvent::LogAppended(_) => {
                cx.notify();
            }
            ConsoleEvent::OperationStarted(_) => {
                if this.gpui_config.log_drawer_open {
                    this.console.update(cx, |c, cx| c.set_open(true, cx));
                    this.console_height_scalar.retarget(
                        this.gpui_config.log_drawer_height,
                        MotionDurations::STANDARD,
                        Instant::now(),
                        this.gpui_config.reduce_motion,
                    );
                }
                cx.notify();
            }
            ConsoleEvent::OperationFinished(status) => {
                let reduce = this.gpui_config.reduce_motion;
                let (kind, title, msg, action) = match status {
                    crate::state::OperationStatus::Success(m) => {
                        (ToastKind::Success, "Opération réussie", m.clone(), None)
                    }
                    crate::state::OperationStatus::Error(m) => (
                        ToastKind::Error,
                        "Échec de l'opération",
                        m.clone(),
                        Some(ToastAction::OpenLogs),
                    ),
                    _ => return,
                };
                if matches!(status, crate::state::OperationStatus::Error(_)) {
                    this.console_height_scalar.retarget(
                        this.gpui_config.log_drawer_height,
                        MotionDurations::EMPHASIS,
                        Instant::now(),
                        this.gpui_config.reduce_motion,
                    );
                }
                this.toast_center.update(cx, |tc, cx| {
                    tc.post(kind, title, msg, action, reduce, cx);
                });
                cx.notify();
            }
            ConsoleEvent::Toggled(is_open) => {
                let target = if *is_open {
                    this.gpui_config.log_drawer_height
                } else {
                    0.0
                };
                this.console_height_scalar.retarget(
                    target,
                    MotionDurations::STANDARD,
                    Instant::now(),
                    this.gpui_config.reduce_motion,
                );
                cx.notify();
            }
            ConsoleEvent::AutoScrollToggled(_) => {
                cx.notify();
            }
            ConsoleEvent::LogsCleared => {
                cx.notify();
            }
        })
        .detach();

        let mut view = Self {
            session,
            store,
            console,
            toast_center,
            workstation,
            settings,
            shelly_settings,
            gpui_config: gpui_config.clone(),
            theme,
            search_focus: cx.focus_handle(),
            search_input_buffer: String::new(),
            search_debounce_task: None,
            news: Vec::new(),
            is_loading_news: false,
            is_mutating: false,
            sidebar_width_scalar,
            console_height_scalar,
            motion_policy: crate::state::motion::MotionPolicy::new(gpui_config.reduce_motion),
            logs_copied_feedback: false,
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
                        "Mise à jour complète du système",
                        MutationAction::UpgradeSystem,
                        "system".to_string(),
                        PackageKey::new(PackageSourceKind::Alpm, "system-upgrade", None),
                        cx,
                    );
                });
            }));
        });

        // Chargement initial asynchrone non-bloquant
        view.trigger_initial_load(cx);

        // Configuration d'environnement pour automatisation / captures d'acceptation
        if let Ok(dest) = std::env::var("SHELLY_NAV_DESTINATION") {
            let parsed_dest = match dest.to_ascii_lowercase().as_str() {
                "settings" => Some(NavDestination::Settings),
                "updates" => Some(NavDestination::Updates),
                "installed" => Some(NavDestination::Installed),
                "news" => Some(NavDestination::News),
                "browse" | "packages" => Some(NavDestination::Browse),
                _ => None,
            };
            if let Some(d) = parsed_dest {
                view.session.update(cx, |s, cx| s.set_destination(d, cx));
            }
        }

        if let Ok(mode) = std::env::var("SHELLY_VIEW_MODE") {
            if mode.eq_ignore_ascii_case("table") {
                view.session.update(cx, |s, cx| {
                    s.set_view_mode(crate::state::PackageViewMode::Table, cx)
                });
            } else if mode.eq_ignore_ascii_case("cards") {
                view.session.update(cx, |s, cx| {
                    s.set_view_mode(crate::state::PackageViewMode::Cards, cx)
                });
            }
        }

        if let Ok(tab) = std::env::var("SHELLY_INSPECTOR_TAB") {
            let parsed_tab = match tab.to_ascii_lowercase().as_str() {
                "dependencies" => Some(InspectorTab::Dependencies),
                "files_build" | "build" | "files" => Some(InspectorTab::FilesBuild),
                "overview" => Some(InspectorTab::Overview),
                _ => None,
            };
            if let Some(t) = parsed_tab {
                view.session.update(cx, |s, cx| s.set_inspector_tab(t, cx));
            }
        }

        if let Ok(collapsed) = std::env::var("SHELLY_SIDEBAR_COLLAPSED") {
            if collapsed == "1" || collapsed.eq_ignore_ascii_case("true") {
                view.session
                    .update(cx, |s, cx| s.set_sidebar_collapsed(true, cx));
                view.sidebar_width_scalar.snap(56.0);
            }
        }

        if let Ok(console_open) = std::env::var("SHELLY_CONSOLE_OPEN") {
            if console_open == "1" || console_open.eq_ignore_ascii_case("true") {
                view.console.update(cx, |c, cx| c.set_open(true, cx));
                view.console_height_scalar.snap(220.0);
            } else if console_open == "0" || console_open.eq_ignore_ascii_case("false") {
                view.console.update(cx, |c, cx| c.set_open(false, cx));
                view.console_height_scalar.snap(0.0);
            }
        }

        if let Ok(width_str) = std::env::var("SHELLY_SPLITTER_WIDTH") {
            if let Ok(w) = width_str.parse::<f32>() {
                view.workstation.update(cx, |ws, _cx| {
                    ws.list_pane_width = w.clamp(280.0, 700.0);
                });
            }
        }

        if let Ok(toast_kind) = std::env::var("SHELLY_TRIGGER_TOAST") {
            let (kind, title, msg, action) = match toast_kind.to_ascii_lowercase().as_str() {
                "error" => (
                    ToastKind::Error,
                    "Échec de l'opération",
                    "Impossible de synchroniser le dépôt cible.",
                    Some(ToastAction::OpenLogs),
                ),
                "warning" => (
                    ToastKind::Warning,
                    "Avertissement",
                    "Le PKGBUILD n'a pu être vérifié en ligne.",
                    None,
                ),
                "info" => (
                    ToastKind::Info,
                    "Information",
                    "Journaux copiés dans le presse-papiers.",
                    None,
                ),
                _ => (
                    ToastKind::Success,
                    "Opération réussie",
                    "Paramètres enregistrés avec succès.",
                    None,
                ),
            };
            let reduce = view.gpui_config.reduce_motion;
            view.toast_center.update(cx, |tc, cx| {
                tc.post(kind, title, msg, action, reduce, cx);
            });
        }

        if let Ok(query) = std::env::var("SHELLY_SEARCH_QUERY") {
            if !query.trim().is_empty() {
                view.search_input_buffer = query.clone();
                view.session.update(cx, |s, _cx| {
                    s.search_query = query.clone();
                });
                view.execute_search(query, cx);
            }
        }

        view
    }

    /// Déclenche le chargement initial en arrière-plan
    fn trigger_initial_load(&self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(updates) = client.list_updates().await {
                let unified: Vec<UnifiedPackage> = updates
                    .into_iter()
                    .map(UnifiedPackage::from_update)
                    .collect();
                let _ = this.update(cx, |view, cx| {
                    view.store.update(cx, |st, cx| {
                        st.set_updates_packages(unified, cx);
                    });
                });
            }
        })
        .detach();

        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(installed) = client.search_installed("").await {
                let unified: Vec<UnifiedPackage> = installed
                    .into_iter()
                    .map(|p| UnifiedPackage::from_alpm(p, true))
                    .collect();
                let _ = this.update(cx, |view, cx| {
                    view.store.update(cx, |st, cx| {
                        st.set_installed_packages(unified, cx);
                    });
                    if let Ok(target) = std::env::var("SHELLY_SELECT_PACKAGE") {
                        if view.session.read(cx).selected_package_key.is_none() {
                            let store = view.store.read(cx);
                            if let Some(pkg) = store
                                .installed_packages
                                .iter()
                                .find(|p| p.name.eq_ignore_ascii_case(&target))
                            {
                                let key = pkg.key();
                                view.session.update(cx, |s, cx| {
                                    s.select_package(Some(key), cx);
                                });
                            }
                        }
                    }
                });
            }
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
            if let Ok(packages) = client.search_installed("").await {
                let unified: Vec<UnifiedPackage> = packages
                    .into_iter()
                    .map(|p| UnifiedPackage::from_alpm(p, true))
                    .collect();
                let _ = this.update(cx, |view, cx| {
                    view.store.update(cx, |st, cx| {
                        st.set_installed_packages(unified, cx);
                    });
                    cx.notify();
                });
            }
        })
        .detach();
    }

    /// Charge les mises à jour disponibles via le backend Shelly
    fn load_updates(&mut self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(packages) = client.list_updates().await {
                let unified: Vec<UnifiedPackage> = packages
                    .into_iter()
                    .map(UnifiedPackage::from_update)
                    .collect();
                let _ = this.update(cx, |view, cx| {
                    view.store.update(cx, |st, cx| {
                        st.set_updates_packages(unified, cx);
                    });
                    cx.notify();
                });
            }
        })
        .detach();
    }

    /// Charge les actualités officielles Arch Linux
    fn load_news(&mut self, cx: &mut Context<Self>) {
        self.is_loading_news = true;
        let client = self.store.read(cx).client.clone();

        cx.spawn(async move |this, cx| {
            let news_items = client.list_news().await.unwrap_or_default();
            let _ = this.update(cx, |view, cx| {
                view.news = news_items;
                view.is_loading_news = false;
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
                if let Ok(Some(details)) = client.get_package_details(&pkg_name).await {
                    let _ = this.update(cx, |view, cx| {
                        view.store.update(cx, |st, _cx| {
                            st.cache_details(key_clone, details);
                        });
                        cx.notify();
                    });
                }
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
                                "PKGBUILD indisponible",
                                format!("Impossible de charger le PKGBUILD pour {}", name_clone),
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

    /// Exécute la recherche unifiée multi-sources avec cache de session et protection contre les requêtes obsolètes
    fn execute_search(&mut self, query: String, cx: &mut Context<Self>) {
        let trimmed = query.trim().to_string();

        if trimmed.is_empty() {
            self.session.update(cx, |s, cx| {
                s.search_query = String::new();
                s.set_searching(false, cx);
                s.select_package(None, cx);
            });
            self.store.update(cx, |st, cx| {
                st.set_active_results(Vec::new(), usize::MAX, cx);
            });
            return;
        }

        let filter = self.session.read(cx).source_filter;

        // 1. Vérification du cache de session
        if let Some(cached) = self
            .store
            .read(cx)
            .get_cached_search(&trimmed, filter)
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

        let client = self.store.read(cx).client.clone();
        let query_clone = trimmed.clone();

        cx.spawn(async move |this, cx| {
            let results: Vec<UnifiedPackage> = match filter {
                SourceFilter::All => {
                    let (alpm_res, aur_res, flatpak_res) = client.search_all(&query_clone).await;
                    let appimage_res = client.list_appimages().await;
                    let mut combined = Vec::new();

                    if let Ok(pkgs) = alpm_res {
                        for p in pkgs {
                            combined.push(UnifiedPackage::from_alpm(p, false));
                        }
                    }
                    if let Ok(pkgs) = aur_res {
                        for p in pkgs {
                            combined.push(UnifiedPackage::from_aur(p, false));
                        }
                    }
                    if let Ok(pkgs) = flatpak_res {
                        for p in pkgs {
                            combined.push(UnifiedPackage::from_flatpak(p, false));
                        }
                    }
                    if let Ok(appimages) = appimage_res {
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
                    combined
                }
                SourceFilter::Alpm => client
                    .search_standard(&query_clone)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| UnifiedPackage::from_alpm(p, false))
                    .collect(),
                SourceFilter::Aur => client
                    .search_aur(&query_clone)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| UnifiedPackage::from_aur(p, false))
                    .collect(),
                SourceFilter::Flatpak => client
                    .search_flatpak(&query_clone)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| UnifiedPackage::from_flatpak(p, false))
                    .collect(),
                SourceFilter::AppImage => {
                    let appimages = client.list_appimages().await.unwrap_or_default();
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
                }
            };

            let _ = this.update(cx, |view, cx| {
                let current_gen = view.session.read(cx).search_generation;
                if current_gen == gen {
                    let mut selected_key_to_set = None;
                    if let Ok(target) = std::env::var("SHELLY_SELECT_PACKAGE") {
                        if let Some(pkg) = results
                            .iter()
                            .find(|p| p.name.eq_ignore_ascii_case(&target))
                        {
                            selected_key_to_set = Some(pkg.key());
                        }
                    }
                    view.store.update(cx, |st, cx| {
                        st.cache_search(&query_clone, filter, results.clone());
                        st.set_active_results(results, gen, cx);
                    });
                    view.session.update(cx, |s, cx| {
                        s.set_searching(false, cx);
                        if let Some(key) = selected_key_to_set {
                            s.select_package(Some(key), cx);
                        }
                    });
                    view.workstation.update(cx, |ws, _cx| {
                        ws.scroll_handle.scroll_to_item(0, ScrollStrategy::Top)
                    });
                    cx.notify();
                }
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
        if self.is_mutating {
            return;
        }

        self.is_mutating = true;
        self.console.update(cx, |c, cx| {
            c.start_operation(action_name, cx);
        });

        if self.gpui_config.log_drawer_open {
            self.console_height_scalar.retarget(
                self.gpui_config.log_drawer_height,
                MotionDurations::STANDARD,
                Instant::now(),
                self.gpui_config.reduce_motion,
            );
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        let client = self.store.read(cx).client.clone();

        match action {
            MutationAction::Install { is_aur, is_flatpak } => {
                client.install_package(&name, is_aur, is_flatpak, tx);
            }
            MutationAction::Remove { is_flatpak } => {
                client.remove_package(&name, is_flatpak, tx);
            }
            MutationAction::UpgradeSystem => {
                client.upgrade_system(tx);
            }
        }

        cx.spawn(async move |this, cx| {
            let mut final_status = false;
            let mut final_msg = String::from("Opération terminée");

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
                            format!("Succès (code {:?})", code)
                        } else {
                            format!("Échec de l'opération (code {:?})", code)
                        };
                    }
                }
            }

            let _ = this.update(cx, |view, cx| {
                view.is_mutating = false;
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
    pub fn save_settings(&mut self, cx: &mut Context<Self>) {
        match self.settings.save() {
            Ok(()) => {
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

                self.workstation.update(cx, |ws, _cx| {
                    ws.theme = theme;
                    ws.reduce_motion = reduce;
                });

                self.toast_center.update(cx, |tc, cx| {
                    tc.post(
                        ToastKind::Success,
                        "Paramètres enregistrés",
                        "Vos préférences ont été sauvegardées sur le disque.",
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
                        "Erreur de sauvegarde",
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
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let packages = self.current_display_packages(cx);
        if packages.is_empty() {
            return;
        }

        let key = event.keystroke.key.as_str();
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

    /// Copie les logs de la console dans le presse-papiers
    pub fn copy_logs_to_clipboard(&mut self, cx: &mut Context<Self>) {
        let logs_text = self
            .console
            .read(cx)
            .logs
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        cx.write_to_clipboard(ClipboardItem::new_string(logs_text));
        self.logs_copied_feedback = true;
        let reduce = self.gpui_config.reduce_motion;
        self.toast_center.update(cx, |tc, cx| {
            tc.post(
                ToastKind::Info,
                "Journaux copiés",
                "Le contenu de la console a été copié dans le presse-papiers.",
                None,
                reduce,
                cx,
            );
        });
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(2))
                .await;
            let _ = this.update(cx, |view, cx| {
                view.logs_copied_feedback = false;
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let entity = cx.entity().clone();

        // ── Animation continue de la barre latérale et du tiroir de logs ─────
        let sidebar_animating = self.sidebar_width_scalar.update(Instant::now());
        let console_animating = self.console_height_scalar.update(Instant::now());
        if sidebar_animating
            || console_animating
            || self.sidebar_width_scalar.is_active()
            || self.console_height_scalar.is_active()
        {
            _window.request_animation_frame();
        }

        let current_sidebar_width = self.sidebar_width_scalar.current;
        let current_console_height = self.console_height_scalar.current;

        // ── Lecture de l'état de session ────────────────────────────────────
        let (destination, is_sidebar_collapsed, destination_epoch) = {
            let session = self.session.read(cx);
            (
                session.destination,
                session.sidebar_collapsed,
                session.destination_epoch,
            )
        };

        let updates_count = self.store.read(cx).updates_count;

        // ── 1. Barre latérale de navigation ─────────────────────────────────
        let entity_dest = entity.clone();
        let entity_toggle = entity.clone();
        let nav_sidebar = Sidebar::render(SidebarProps {
            active_destination: destination,
            updates_count,
            is_collapsed: is_sidebar_collapsed,
            current_width: current_sidebar_width,
            theme: &theme,
            on_select_destination: Rc::new(move |dest, _w, cx| {
                entity_dest.update(cx, |view, cx| {
                    view.session.update(cx, |s, cx| s.set_destination(dest, cx));
                });
            }),
            on_toggle_collapse: Rc::new(move |_w, cx| {
                entity_toggle.update(cx, |view, cx| {
                    view.session.update(cx, |s, cx| s.toggle_sidebar(cx));
                });
            }),
        });

        // ── 2. Contenu principal selon la destination ───────────────────────
        let main_content = match destination {
            NavDestination::Browse | NavDestination::Installed | NavDestination::Updates => {
                div().size_full().child(self.workstation.clone())
            }
            NavDestination::News => div().size_full().child(NewsView::render(NewsViewProps {
                news: &self.news,
                is_loading: self.is_loading_news,
                theme: &theme,
            })),
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
                        on_toggle_shelly_search: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_shelly_search();
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
                        on_toggle_no_confirm: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_no_confirm();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_dark_theme: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_dark_theme();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_compact_view: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_compact_view();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_log_drawer_auto_open: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_log_drawer_auto_open();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_reduce_motion: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_reduce_motion();
                                    cx.notify();
                                })
                            })
                        },
                        on_save: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.save_settings(cx);
                                })
                            })
                        },
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

        // ── 3. Tiroir de console d'opérations avec transition continue ───────
        let entity_log_toggle = entity.clone();
        let entity_log_copy = entity.clone();
        let entity_log_clear = entity.clone();
        let entity_log_auto = entity.clone();

        let console_read = self.console.read(cx);
        let log_drawer = LogDrawer::render(LogDrawerProps {
            logs: &console_read.logs,
            status: &console_read.status,
            is_open: console_read.is_open,
            auto_scroll: console_read.auto_scroll,
            height: current_console_height,
            copied_feedback: self.logs_copied_feedback,
            scroll_handle: &console_read.scroll_handle,
            theme: &theme,
            on_toggle: Some(Rc::new(move |_e, _w, cx| {
                entity_log_toggle.update(cx, |view, cx| {
                    view.console.update(cx, |c, cx| c.toggle_drawer(cx));
                });
            })),
            on_copy: Some(Rc::new(move |_e, _w, cx| {
                entity_log_copy.update(cx, |view, cx| view.copy_logs_to_clipboard(cx));
            })),
            on_clear: Some(Rc::new(move |_e, _w, cx| {
                entity_log_clear.update(cx, |view, cx| {
                    view.console.update(cx, |c, cx| c.clear_logs(cx))
                });
            })),
            on_toggle_autoscroll: Some(Rc::new(move |_e, _w, cx| {
                entity_log_auto.update(cx, |view, cx| {
                    view.console.update(cx, |c, cx| c.toggle_auto_scroll(cx))
                });
            })),
        });

        // ── 4. Overlay de notifications (Toasts) ────────────────────────────
        let toasts = self.toast_center.read(cx).toasts.clone();
        let entity_toast_dismiss = entity.clone();
        let entity_toast_action = entity.clone();

        let toast_overlay = ToastOverlay::render(ToastOverlayProps {
            toasts: &toasts,
            theme: &theme,
            on_dismiss: Rc::new(move |id, _w, cx| {
                entity_toast_dismiss.update(cx, |view, cx| {
                    view.toast_center.update(cx, |tc, cx| {
                        tc.remove_toast(id);
                        cx.notify();
                    });
                });
            }),
            on_action: Rc::new(move |action, _w, cx| {
                entity_toast_action.update(cx, |view, cx| match action {
                    ToastAction::OpenLogs => {
                        view.console.update(cx, |c, cx| c.set_open(true, cx));
                        view.console_height_scalar.retarget(
                            view.gpui_config.log_drawer_height,
                            MotionDurations::STANDARD,
                            Instant::now(),
                            view.gpui_config.reduce_motion,
                        );
                        cx.notify();
                    }
                });
            }),
        });

        // ── 5. Assemblage final du shell ─────────────────────────────────────
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
            .child(nav_sidebar)
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
                    .child(log_drawer),
            )
            .child(toast_overlay)
    }
}
