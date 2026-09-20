use crate::backend::client::ShellyClient;
use crate::backend::models::{ArchNewsItem, UnifiedPackage};
use crate::backend::process::LogStreamEvent;
use crate::components::log_drawer::{LogDrawer, LogDrawerProps};
use crate::components::package_card::{PackageCard, PackageCardProps};
use crate::components::package_table::PackageTable;
use crate::components::sidebar::{Sidebar, SidebarProps};
use crate::components::unified_search::{UnifiedSearch, UnifiedSearchProps};
use crate::config::{ConfigManager, GpuiUiConfig, ShellySettings};
use crate::state::{
    AppSession, ConsoleEvent, ConsoleModel, InspectorTab, NavDestination, PackageKey,
    PackageSourceKind, PackageStore, PackageStoreEvent, PackageViewMode, SessionEvent,
    SourceFilter,
};
use crate::theme::Theme;
use crate::views::inspector::{PackageInspectorProps, PackageInspectorView};
use crate::views::news::{NewsView, NewsViewProps};
use crate::views::settings::{SettingsView, SettingsViewProps};
use gpui::*;
use gpui::{uniform_list, ScrollStrategy, UniformListScrollHandle};
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
    pub shelly_settings: ShellySettings,
    pub gpui_config: GpuiUiConfig,
    pub theme: Theme,
    pub search_focus: FocusHandle,
    pub search_input_buffer: String,
    pub search_debounce_task: Option<Task<()>>,
    pub news: Vec<ArchNewsItem>,
    pub is_loading_news: bool,
    pub is_mutating: bool,
    pub list_pane_width: f32,
    pub is_resizing_splitter: bool,
    pub scroll_handle: UniformListScrollHandle,
    pub logs_copied_feedback: bool,
    pub copy_cmd_feedback: bool,
    pub is_loading_pkgbuild: bool,
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
        let session = cx.new(|_cx| AppSession::new());
        let store = cx.new(|_cx| PackageStore::new(client));
        let console = cx.new(|_cx| ConsoleModel::new());

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

        cx.subscribe(&console, |_this, _emitter, event, cx| match event {
            ConsoleEvent::LogAppended(_) => {
                cx.notify();
            }
            ConsoleEvent::OperationStarted(_) => {
                cx.notify();
            }
            ConsoleEvent::OperationFinished(_) => {
                cx.notify();
            }
            ConsoleEvent::Toggled(_) => {
                cx.notify();
            }
            ConsoleEvent::LogsCleared => {
                cx.notify();
            }
            ConsoleEvent::AutoScrollToggled(_) => {
                cx.notify();
            }
        })
        .detach();

        let view = Self {
            session,
            store,
            console,
            shelly_settings,
            gpui_config,
            theme,
            search_focus: cx.focus_handle(),
            search_input_buffer: String::new(),
            search_debounce_task: None,
            news: Vec::new(),
            is_loading_news: false,
            is_mutating: false,
            list_pane_width: 460.0,
            is_resizing_splitter: false,
            scroll_handle: UniformListScrollHandle::new(),
            logs_copied_feedback: false,
            copy_cmd_feedback: false,
            is_loading_pkgbuild: false,
        };

        // Chargement initial asynchrone non-bloquant
        view.trigger_initial_load(cx);

        view
    }

    /// Déclenche le chargement concurrent des données initiales (mises à jour & inventaire local)
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
                });
            }
        })
        .detach();
    }

    /// Réaction au changement de destination
    fn on_destination_changed(&mut self, dest: NavDestination, cx: &mut Context<Self>) {
        self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);

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
                if self.news.is_empty() && !self.is_loading_news {
                    self.load_news(cx);
                }
            }
            NavDestination::Settings => {}
        }
    }

    /// Charge les paquets installés localement
    fn load_installed_packages(&mut self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(pkgs) = client.search_installed("").await {
                let unified: Vec<UnifiedPackage> = pkgs
                    .into_iter()
                    .map(|p| UnifiedPackage::from_alpm(p, true))
                    .collect();
                let _ = this.update(cx, |view, cx| {
                    view.store.update(cx, |st, cx| {
                        st.set_installed_packages(unified, cx);
                    });
                });
            }
        })
        .detach();
    }

    /// Charge les mises à jour disponibles
    fn load_updates(&mut self, cx: &mut Context<Self>) {
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
    }

    /// Charge les actualités Arch Linux
    fn load_news(&mut self, cx: &mut Context<Self>) {
        self.is_loading_news = true;
        let client = self.store.read(cx).client.clone();

        cx.spawn(async move |this, cx| {
            let news = client.list_news().await.unwrap_or_default();
            let _ = this.update(cx, |view, cx| {
                view.news = news;
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

        self.is_loading_pkgbuild = true;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let res = client.fetch_aur_pkgbuild(&name_clone).await;
            let _ = this.update(cx, |view, cx| {
                view.is_loading_pkgbuild = false;
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
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Copie la commande canonique d'installation dans le presse-papiers avec feedback visuel
    pub fn copy_install_command(&mut self, cmd: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(cmd));
        self.copy_cmd_feedback = true;
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(2))
                .await;
            let _ = this.update(cx, |view, cx| {
                view.copy_cmd_feedback = false;
                cx.notify();
            });
        })
        .detach();
    }

    /// Copie le PKGBUILD dans le presse-papiers
    pub fn copy_pkgbuild_to_clipboard(&mut self, content: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(content));
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
                s.set_search_query(String::new(), cx);
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
            let gen = self.session.update(cx, |s, cx| {
                s.set_search_query(trimmed.clone(), cx);
                s.set_searching(false, cx);
                s.next_search_generation()
            });
            self.store.update(cx, |st, cx| {
                st.set_active_results(cached, gen, cx);
            });
            self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
            return;
        }

        // 2. Requête backend concurrente
        let gen = self.session.update(cx, |s, cx| {
            s.set_search_query(trimmed.clone(), cx);
            s.set_searching(true, cx);
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
                    view.store.update(cx, |st, cx| {
                        st.cache_search(&query_clone, filter, results.clone());
                        st.set_active_results(results, gen, cx);
                    });
                    view.session.update(cx, |s, cx| {
                        s.set_searching(false, cx);
                    });
                    view.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
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
                    // Invalidation chirurgicale des caches
                    view.store.update(cx, |st, cx| {
                        st.invalidate_package(&key, cx);
                        st.invalidate_installed(cx);
                        st.invalidate_updates(cx);
                    });

                    // Rafraîchit les données selon la vue courante
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
                self.scroll_handle
                    .scroll_to_item_strict(next_idx, ScrollStrategy::Top);
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
                self.scroll_handle
                    .scroll_to_item_strict(prev_idx, ScrollStrategy::Top);
            }
            "escape" => {
                self.session.update(cx, |s, cx| {
                    s.select_package(None, cx);
                });
            }
            _ => {}
        }
    }

    /// Gestion du redimensionnement du panneau latéral par le splitter
    pub fn on_splitter_pointer_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if self.is_resizing_splitter {
            let new_width = event.position.x.to_f64() as f32;
            let clamped = new_width.clamp(280.0, 700.0);
            if (clamped - self.list_pane_width).abs() >= 1.0 {
                self.list_pane_width = clamped;
                cx.notify();
            }
        }
    }

    pub fn on_splitter_pointer_up(&mut self, _event: &MouseUpEvent, _cx: &mut Context<Self>) {
        if self.is_resizing_splitter {
            self.is_resizing_splitter = false;
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

        // ── Lecture de l'état de session ────────────────────────────────────
        let (
            destination,
            source_filter,
            is_searching,
            selected_key,
            is_sidebar_collapsed,
            view_mode,
            inspector_tab,
        ) = {
            let session = self.session.read(cx);
            (
                session.destination,
                session.source_filter,
                session.is_searching,
                session.selected_package_key.clone(),
                session.sidebar_collapsed,
                session.view_mode,
                session.inspector_tab,
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
                let packages = self.current_display_packages(cx).to_vec();
                let selected_pkg = selected_key
                    .as_ref()
                    .and_then(|sk| packages.iter().find(|p| &p.key() == sk).cloned());

                // En-tête de recherche ou titre de vue
                let top_bar = match destination {
                    NavDestination::Browse => {
                        let entity_filter = entity.clone();
                        let entity_mode = entity.clone();
                        let filter_bar = UnifiedSearch::render_filter_bar(&UnifiedSearchProps {
                            active_filter: source_filter,
                            is_searching,
                            total_count: packages.len(),
                            view_mode,
                            theme: &theme,
                            on_select_filter: Rc::new(move |filter, _w, cx| {
                                entity_filter.update(cx, |view, cx| {
                                    view.session
                                        .update(cx, |s, cx| s.set_source_filter(filter, cx));
                                });
                            }),
                            on_select_view_mode: Rc::new(move |mode, _w, cx| {
                                entity_mode.update(cx, |view, cx| {
                                    view.session.update(cx, |s, cx| s.set_view_mode(mode, cx));
                                });
                            }),
                        });

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
                            .child(div().text_sm().child("🔍"))
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(theme.text_primary)
                                    .child(if self.search_input_buffer.is_empty() {
                                        div()
                                            .text_color(theme.text_muted)
                                            .child("Search packages (e.g. ripgrep, firefox)...")
                                    } else {
                                        div().child(self.search_input_buffer.clone())
                                    }),
                            )
                            .on_key_down(move |event, _w, cx| {
                                let key = event.keystroke.key.as_str();
                                entity_input.update(cx, |view, cx| {
                                    if key == "backspace" {
                                        let mut cur = view.search_input_buffer.clone();
                                        cur.pop();
                                        view.on_search_input(cur, cx);
                                    } else if key == "escape" {
                                        view.on_search_input(String::new(), cx);
                                    } else if event.keystroke.key.chars().count() == 1 {
                                        let mut cur = view.search_input_buffer.clone();
                                        cur.push_str(key);
                                        view.on_search_input(cur, cx);
                                    }
                                });
                            });

                        div()
                            .flex()
                            .flex_col()
                            .child(search_input)
                            .child(filter_bar)
                    }
                    NavDestination::Installed => div()
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
                                .font_weight(FontWeight::BOLD)
                                .text_sm()
                                .text_color(theme.text_primary)
                                .child("📦 Locally Installed Packages"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(format!("{} packages", packages.len())),
                        ),
                    NavDestination::Updates => {
                        let entity_upgrade = entity.clone();
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
                                    .font_weight(FontWeight::BOLD)
                                    .text_sm()
                                    .text_color(theme.text_primary)
                                    .child("🔄 Available System Updates"),
                            )
                            .child(
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
                                    .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                                        entity_upgrade.update(cx, |view, cx| {
                                            view.run_package_mutation(
                                                "Mise à jour complète du système",
                                                MutationAction::UpgradeSystem,
                                                "system".to_string(),
                                                PackageKey::new(
                                                    PackageSourceKind::Alpm,
                                                    "system-upgrade",
                                                    None,
                                                ),
                                                cx,
                                            );
                                        });
                                    }),
                            )
                    }
                    _ => div(),
                };

                // Corps de la liste : Si recherche vide dans Browse -> état de découverte
                let is_browse_empty = destination == NavDestination::Browse
                    && self.search_input_buffer.trim().is_empty();

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
                        .child(div().text_sm().text_color(theme.text_muted).child(
                            if is_searching {
                                "Searching..."
                            } else {
                                "No matching packages."
                            },
                        ))
                        .into_any_element()
                } else {
                    let package_count = packages.len();
                    let entity_select = entity.clone();
                    let list_theme = theme;
                    let selected_name = selected_pkg.as_ref().map(|p| p.name.clone());

                    if view_mode == PackageViewMode::Table {
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
                                                        .h(px(36.0))
                                                        .child(PackageTable::render_row(
                                                            pkg,
                                                            is_selected,
                                                            &list_theme,
                                                        ))
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            move |_e, _w, cx| {
                                                                let key = pkg_key.clone();
                                                                on_select.update(cx, |view, cx| {
                                                                    view.session.update(
                                                                        cx,
                                                                        |s, cx| {
                                                                            s.select_package(
                                                                                Some(key),
                                                                                cx,
                                                                            );
                                                                        },
                                                                    );
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
                                            .h(px(78.0))
                                            .px_3()
                                            .py_1()
                                            .child(PackageCard::render(PackageCardProps {
                                                package: pkg,
                                                is_selected,
                                                theme: &list_theme,
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

                // Splitter draggable
                let entity_split = entity.clone();
                let splitter = div()
                    .w(px(5.0))
                    .h_full()
                    .bg(if self.is_resizing_splitter {
                        theme.accent
                    } else {
                        theme.border
                    })
                    .cursor_col_resize()
                    .hover({
                        let h = theme.accent;
                        move |s| s.bg(h)
                    })
                    .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                        entity_split.update(cx, |view, cx| {
                            view.is_resizing_splitter = true;
                            cx.notify();
                        });
                    });

                // Panneau d'inspection sémantique
                let entity_install = entity.clone();
                let entity_remove = entity.clone();
                let entity_tab = entity.clone();
                let entity_copy_cmd = entity.clone();
                let entity_copy_pkgbuild = entity.clone();
                let entity_nav_dep = entity.clone();

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
                let details_pane =
                    div()
                        .flex_1()
                        .h_full()
                        .overflow_hidden()
                        .child(PackageInspectorView::render(PackageInspectorProps {
                            package: selected_pkg.as_ref(),
                            alpm_details: alpm_details.as_ref(),
                            pkgbuild: cached_pkgbuild,
                            is_loading_pkgbuild: self.is_loading_pkgbuild,
                            active_tab: inspector_tab,
                            theme: &theme,
                            is_busy: self.is_mutating,
                            copy_feedback: self.copy_cmd_feedback,
                            on_select_tab: Rc::new(move |tab, _w, cx| {
                                entity_tab.update(cx, |view, cx| {
                                    view.session.update(cx, |s, cx| {
                                        s.set_inspector_tab(tab, cx);
                                    });
                                });
                            }),
                            on_install: selected_pkg_clone.as_ref().map(|p| {
                                let p_clone = p.clone();
                                let key = p.key();
                                let is_aur = p.source_type == "AUR";
                                let is_flatpak = p.source_type == "Flatpak";
                                Rc::new(
                                    move |_e: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                                        let key_c = key.clone();
                                        let pkg_name = p_clone.name.clone();
                                        entity_install.update(cx, |view, cx| {
                                            view.run_package_mutation(
                                                &format!("Installation of {}", pkg_name),
                                                MutationAction::Install { is_aur, is_flatpak },
                                                pkg_name,
                                                key_c,
                                                cx,
                                            );
                                        });
                                    },
                                )
                                    as Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>
                            }),
                            on_remove: selected_pkg_clone.as_ref().map(|p| {
                                let p_clone = p.clone();
                                let key = p.key();
                                let is_flatpak = p.source_type == "Flatpak";
                                Rc::new(
                                    move |_e: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                                        let key_c = key.clone();
                                        let pkg_name = p_clone.name.clone();
                                        entity_remove.update(cx, |view, cx| {
                                            view.run_package_mutation(
                                                &format!("Uninstallation of {}", pkg_name),
                                                MutationAction::Remove { is_flatpak },
                                                pkg_name,
                                                key_c,
                                                cx,
                                            );
                                        });
                                    },
                                )
                                    as Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>
                            }),
                            on_copy_install_cmd: selected_pkg_clone.as_ref().and_then(|p| {
                                crate::state::canonical_install_command(p).map(|cmd| {
                                    Rc::new(move |_w: &mut Window, cx: &mut App| {
                                        let cmd_c = cmd.clone();
                                        entity_copy_cmd.update(cx, |view, cx| {
                                            view.copy_install_command(cmd_c, cx);
                                        });
                                    })
                                        as Rc<dyn Fn(&mut Window, &mut App)>
                                })
                            }),
                            on_copy_pkgbuild: cached_pkgbuild.cloned().map(|content| {
                                Rc::new(move |_w: &mut Window, cx: &mut App| {
                                    let content_c = content.clone();
                                    entity_copy_pkgbuild.update(cx, |view, cx| {
                                        view.copy_pkgbuild_to_clipboard(content_c, cx);
                                    });
                                })
                                    as Rc<dyn Fn(&mut Window, &mut App)>
                            }),
                            on_navigate_package: Some(Rc::new(move |dep_name, _w, cx| {
                                entity_nav_dep.update(cx, |view, cx| {
                                    view.session.update(cx, |s, cx| {
                                        s.set_destination(NavDestination::Browse, cx);
                                    });
                                    view.on_search_input(dep_name, cx);
                                });
                            })),
                        }));

                div()
                    .flex()
                    .flex_row()
                    .size_full()
                    .child(list_pane)
                    .child(splitter)
                    .child(details_pane)
            }
            NavDestination::News => div().size_full().child(NewsView::render(NewsViewProps {
                news: &self.news,
                is_loading: self.is_loading_news,
                theme: &theme,
            })),
            NavDestination::Settings => {
                div()
                    .size_full()
                    .child(SettingsView::render(SettingsViewProps {
                        shelly_settings: &self.shelly_settings,
                        gpui_config: &self.gpui_config,
                        theme: &theme,
                    }))
            }
        };

        // ── 3. Tiroir de console d'opérations avec auto-scroll réel ──────────
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
            height: console_read.height,
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

        // ── 4. Assemblage final du shell de la station de travail ────────────
        let entity_move = entity.clone();
        let entity_up = entity.clone();
        let entity_key = entity.clone();

        div()
            .id("workspace_root")
            .size_full()
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
            .on_mouse_move(move |event, _window, cx| {
                entity_move.update(cx, |view, cx| {
                    view.on_splitter_pointer_move(event, cx);
                });
            })
            .on_mouse_up(MouseButton::Left, move |event, _window, cx| {
                entity_up.update(cx, |view, cx| {
                    view.on_splitter_pointer_up(event, cx);
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
                            .child(main_content),
                    )
                    .child(log_drawer),
            )
    }
}
