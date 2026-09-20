use crate::backend::client::ShellyClient;
use crate::backend::models::{ArchNewsItem, UnifiedPackage};
use crate::backend::process::LogStreamEvent;
use crate::components::diagnostics_hud::{DiagnosticsHud, DiagnosticsHudProps};
use crate::components::filter_pills::{FilterPills, FilterPillsProps};
use crate::components::log_drawer::{LogDrawer, LogDrawerProps};
use crate::components::nav_rail::{NavRail, NavRailProps};
use crate::components::package_card::{PackageCard, PackageCardProps};
use crate::components::package_table::{
    PackageTable, PackageTableProps, SortColumn, SortDirection,
};
use crate::config::{ConfigManager, GpuiUiConfig, ShellySettings};
use crate::models::{
    CatalogEvent, CatalogModel, ConsoleEvent, ConsoleModel, LayoutEvent, LayoutModel, NavRoute,
    PackageViewMode,
};
use crate::theme::Theme;
use crate::views::details::{PackageDetailsProps, PackageDetailsView};
use crate::views::news::{NewsView, NewsViewProps};
use crate::views::settings::{SettingsView, SettingsViewProps};
use gpui::prelude::FluentBuilder;
use gpui::{uniform_list, ScrollStrategy, UniformListScrollHandle};
use gpui::*;
use std::rc::Rc;
use std::time::Instant;
use tokio::sync::mpsc;

pub struct WorkspaceView {
    pub client: ShellyClient,
    pub shelly_settings: ShellySettings,
    pub gpui_config: GpuiUiConfig,
    pub theme: Theme,
    pub catalog: Entity<CatalogModel>,
    pub console: Entity<ConsoleModel>,
    pub layout: Entity<LayoutModel>,
    pub search_focus: FocusHandle,
    pub search_generation: u64,
    pub news: Vec<ArchNewsItem>,
    pub is_loading_news: bool,
    pub file_list: Vec<String>,
    pub is_resizing_pane: bool,
    pub scroll_handle: UniformListScrollHandle,
    pub table_scroll_handle: UniformListScrollHandle,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub last_render: Option<Instant>,
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

        let catalog = cx.new(|_cx| CatalogModel::new());
        let console = cx.new(|_cx| ConsoleModel::new());
        let layout = cx.new(|_cx| LayoutModel::new());

        // Abonnements typés aux événements des modèles découplés
        cx.subscribe(&catalog, |this, _emitter, event, cx| match event {
            CatalogEvent::SearchStarted { query } => {
                log::debug!("Événement catalogue: Recherche démarrée: '{}'", query);
                this.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                cx.notify();
            }
            CatalogEvent::SearchResultsUpdated {
                total,
                official,
                aur,
                flatpak,
            } => {
                log::debug!(
                    "Événement catalogue: Résultats MAJ (total: {}, officiel: {}, AUR: {}, Flatpak: {})",
                    total,
                    official,
                    aur,
                    flatpak
                );
                this.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                cx.notify();
            }
            CatalogEvent::PackageSelected(pkg_opt) => {
                if let Some(pkg) = pkg_opt {
                    let name = pkg.name.clone();
                    let is_installed = pkg.is_installed;
                    if pkg.source_type == "ALPM" {
                        let cached = this.catalog.read(cx).detail_cache.get(&name).cloned();
                        if cached.is_none() {
                            let client = this.client.clone();
                            let name_clone = name.clone();
                            cx.spawn(async move |this, cx| {
                                if let Ok(Some(details)) = client.get_package_details(&name_clone).await {
                                    let _ = this.update(cx, |view, cx| {
                                        view.catalog.update(cx, |cat, cx| {
                                            cat.detail_cache.insert(name_clone, details);
                                            cx.notify();
                                        });
                                    });
                                }
                            })
                            .detach();
                        }
                    }
                    if is_installed {
                        let client = this.client.clone();
                        cx.spawn(async move |this, cx| {
                            let files = client.list_package_files(&name).await;
                            let _ = this.update(cx, |view, cx| {
                                view.file_list = files;
                                cx.notify();
                            });
                        })
                        .detach();
                    } else {
                        this.file_list.clear();
                    }
                } else {
                    this.file_list.clear();
                }
                cx.notify();
            }
            CatalogEvent::SourceFilterChanged(filter) => {
                log::debug!("Événement catalogue: Filtre de source: {:?}", filter);
                this.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                cx.notify();
            }
            CatalogEvent::UpdatesLoaded(count) => {
                log::debug!("Événement catalogue: Mises à jour chargées: {}", count);
                cx.notify();
            }
            CatalogEvent::CacheInvalidated => {
                log::debug!("Événement catalogue: Cache invalidé");
                cx.notify();
            }
        })
        .detach();

        cx.subscribe(&console, |_this, _emitter, event, cx| match event {
            ConsoleEvent::LogAppended(entry) => {
                let _ = &entry.text;
                cx.notify();
            }
            ConsoleEvent::OperationStarted(title) => {
                log::info!("Événement console: Démarrage: {}", title);
                cx.notify();
            }
            ConsoleEvent::OperationFinished { success, message } => {
                log::info!(
                    "Événement console: Terminé (succès: {}): {}",
                    success,
                    message
                );
                cx.notify();
            }
            ConsoleEvent::Toggled(is_open) => {
                log::debug!("Événement console: Tiroir ouvert = {}", is_open);
                cx.notify();
            }
            ConsoleEvent::LogsCleared => {
                log::debug!("Événement console: Logs effacés");
                cx.notify();
            }
            ConsoleEvent::AutoScrollToggled(enabled) => {
                log::debug!("Événement console: Autoscroll = {}", enabled);
                cx.notify();
            }
        })
        .detach();

        cx.subscribe(&layout, |_this, _emitter, event, cx| match event {
            LayoutEvent::RouteChanged(route) => {
                log::debug!("Événement disposition: Route = {:?}", route);
                cx.notify();
            }
            LayoutEvent::SidebarToggled(collapsed) => {
                log::debug!("Événement disposition: Sidebar repliée = {}", collapsed);
                cx.notify();
            }
            LayoutEvent::ViewModeChanged(mode) => {
                log::debug!("Événement disposition: Mode affichage = {:?}", mode);
                cx.notify();
            }
            LayoutEvent::InspectorTabChanged(tab) => {
                log::debug!("Événement disposition: Onglet inspecteur = {:?}", tab);
                cx.notify();
            }
            LayoutEvent::PaneResized(width) => {
                log::trace!("Événement disposition: Largeur panneau = {}", width);
                cx.notify();
            }
            LayoutEvent::DiagnosticsHudToggled(shown) => {
                log::debug!("Événement disposition: HUD diagnostics = {}", shown);
                cx.notify();
            }
            LayoutEvent::MetricsUpdated { fps, frame_time_ms } => {
                log::trace!("Événement disposition: Métriques: FPS={:.1}, frame={:.1}ms", fps, frame_time_ms);
                cx.notify();
            }
        })
        .detach();

        let mut view = Self {
            client: ShellyClient::default(),
            shelly_settings,
            gpui_config,
            theme,
            catalog,
            console,
            layout,
            search_focus: cx.focus_handle(),
            search_generation: 0,
            news: Vec::new(),
            is_loading_news: false,
            file_list: Vec::new(),
            is_resizing_pane: false,
            scroll_handle: UniformListScrollHandle::new(),
            table_scroll_handle: UniformListScrollHandle::new(),
            sort_column: SortColumn::Name,
            sort_direction: SortDirection::Ascending,
            last_render: None,
            logs_copied_feedback: false,
        };

        view.load_initial_data(cx);
        view
    }

    pub fn load_initial_data(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();

        cx.spawn(async move |this, cx| {
            if let Ok(updates) = client.list_updates().await {
                let _ = this.update(cx, |view, cx| {
                    view.catalog.update(cx, |cat, cx| {
                        cat.set_updates(updates, cx);
                    });
                });
            }

            if let Ok(installed) = client.search_installed("").await {
                let _ = this.update(cx, |view, cx| {
                    view.catalog.update(cx, |cat, cx| {
                        cat.set_search_results(installed, Vec::new(), Vec::new(), cx);
                    });
                });
            }
        })
        .detach();
    }

    pub fn switch_route(&mut self, route: NavRoute, cx: &mut Context<Self>) {
        self.layout.update(cx, |l, cx| {
            l.set_route(route, cx);
        });

        match route {
            NavRoute::Search => {
                let q = self.catalog.read(cx).search_query.clone();
                self.perform_search(q, cx);
            }
            NavRoute::Updates => self.load_updates(cx),
            NavRoute::Installed => self.load_installed(cx),
            NavRoute::Settings => self.load_news(cx),
        }
    }

    pub fn perform_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.catalog.update(cx, |cat, cx| {
            cat.set_search_query(query.clone(), cx);
        });

        let client = self.client.clone();
        let q = query.clone();

        cx.spawn(async move |this, cx| {
            if q.trim().is_empty() {
                if let Ok(installed) = client.search_installed("").await {
                    let _ = this.update(cx, |view, cx| {
                        view.catalog.update(cx, |cat, cx| {
                            cat.set_search_results(installed, Vec::new(), Vec::new(), cx);
                        });
                    });
                }
            } else {
                let (off_res, aur_res, fp_res) = client.search_all(&q).await;
                let off = off_res.unwrap_or_default();
                let aur = aur_res.unwrap_or_default();
                let fp = fp_res.unwrap_or_default();
                let _ = this.update(cx, |view, cx| {
                    view.catalog.update(cx, |cat, cx| {
                        cat.set_search_results(off, aur, fp, cx);
                    });
                });
            }
        })
        .detach();
    }

    pub fn load_installed(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(installed) = client.search_installed("").await {
                let _ = this.update(cx, |view, cx| {
                    view.catalog.update(cx, |cat, cx| {
                        cat.set_search_results(installed, Vec::new(), Vec::new(), cx);
                    });
                });
            }
        })
        .detach();
    }

    pub fn load_updates(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(updates) = client.list_updates().await {
                let _ = this.update(cx, |view, cx| {
                    view.catalog.update(cx, |cat, cx| {
                        cat.set_updates(updates, cx);
                    });
                });
            }
        })
        .detach();
    }

    pub fn load_news(&mut self, cx: &mut Context<Self>) {
        if !self.news.is_empty() {
            return;
        }
        self.is_loading_news = true;
        cx.notify();

        let client = self.client.clone();
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

    pub fn select_package_item(&mut self, pkg: UnifiedPackage, cx: &mut Context<Self>) {
        self.catalog.update(cx, |cat, cx| {
            cat.select_package(Some(pkg), cx);
        });
    }

    pub fn change_sort(&mut self, col: SortColumn, cx: &mut Context<Self>) {
        if self.sort_column == col {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_column = col;
            self.sort_direction = SortDirection::Ascending;
        }
        cx.notify();
    }

    pub fn get_sorted_items(&self, cx: &App) -> Vec<UnifiedPackage> {
        let mut items = self.catalog.read(cx).filtered_items();
        let dir = self.sort_direction;
        match self.sort_column {
            SortColumn::Name => {
                items.sort_by(|a, b| {
                    let cmp = a.name.to_lowercase().cmp(&b.name.to_lowercase());
                    if dir == SortDirection::Ascending { cmp } else { cmp.reverse() }
                });
            }
            SortColumn::Version => {
                items.sort_by(|a, b| {
                    let cmp = a.version.cmp(&b.version);
                    if dir == SortDirection::Ascending { cmp } else { cmp.reverse() }
                });
            }
            SortColumn::Source => {
                items.sort_by(|a, b| {
                    let cmp = a.source_type.cmp(&b.source_type);
                    if dir == SortDirection::Ascending { cmp } else { cmp.reverse() }
                });
            }
            SortColumn::Status => {
                items.sort_by(|a, b| {
                    let cmp = a.is_installed.cmp(&b.is_installed);
                    if dir == SortDirection::Ascending { cmp } else { cmp.reverse() }
                });
            }
        }
        items
    }

    pub fn install_selected(&mut self, cx: &mut Context<Self>) {
        let Some(pkg) = self.catalog.read(cx).selected_package.clone() else {
            return;
        };

        let name = pkg.name.clone();
        let is_aur = pkg.source_type == "AUR";
        let is_flatpak = pkg.source_type == "Flatpak";

        self.console.update(cx, |c, cx| {
            c.clear_logs(cx);
            c.start_operation(format!("Installation de {}", name), cx);
            c.append_stdout(format!(">>> Démarrage de l'installation de {}...", name), cx);
        });

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        self.client.install_package(&name, is_aur, is_flatpak, tx);
        let console_model = self.console.clone();

        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.recv().await {
                match event {
                    LogStreamEvent::Line(line) => {
                        let _ = console_model.update(cx, |c, cx| c.append_stdout(line, cx));
                    }
                    LogStreamEvent::ErrorLine(err) => {
                        let _ = console_model.update(cx, |c, cx| c.append_stderr(err, cx));
                    }
                    LogStreamEvent::Finished(success, code) => {
                        let _ = this.update(cx, |view, cx| {
                            view.console.update(cx, |c, cx| {
                                if success {
                                    c.finish_operation(true, format!("{} installé avec succès", name), cx);
                                    c.append_stdout(">>> Opération terminée avec succès.", cx);
                                } else if code == Some(126) || code == Some(127) {
                                    c.finish_operation(false, "Authentification Polkit annulée", cx);
                                    c.append_stderr(">>> Opération annulée : invite d'authentification fermée ou refusée.", cx);
                                } else {
                                    c.finish_operation(false, format!("Échec de l'installation (code {:?})", code), cx);
                                    c.append_stderr(format!(">>> Erreur critique (code {:?})", code), cx);
                                }
                            });
                            if success {
                                view.refresh_after_operation(cx);
                            }
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn remove_selected(&mut self, cx: &mut Context<Self>) {
        let Some(pkg) = self.catalog.read(cx).selected_package.clone() else {
            return;
        };

        let name = pkg.name.clone();
        let is_flatpak = pkg.source_type == "Flatpak";

        self.console.update(cx, |c, cx| {
            c.clear_logs(cx);
            c.start_operation(format!("Suppression de {}", name), cx);
            c.append_stdout(format!(">>> Suppression du paquet {}...", name), cx);
        });

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        self.client.remove_package(&name, is_flatpak, tx);
        let console_model = self.console.clone();

        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.recv().await {
                match event {
                    LogStreamEvent::Line(line) => {
                        let _ = console_model.update(cx, |c, cx| c.append_stdout(line, cx));
                    }
                    LogStreamEvent::ErrorLine(err) => {
                        let _ = console_model.update(cx, |c, cx| c.append_stderr(err, cx));
                    }
                    LogStreamEvent::Finished(success, code) => {
                        let _ = this.update(cx, |view, cx| {
                            view.console.update(cx, |c, cx| {
                                if success {
                                    c.finish_operation(true, format!("{} désinstallé", name), cx);
                                    c.append_stdout(">>> Désinstallation terminée avec succès.", cx);
                                } else if code == Some(126) || code == Some(127) {
                                    c.finish_operation(false, "Authentification Polkit annulée", cx);
                                    c.append_stderr(">>> Opération annulée : invite d'authentification refusée.", cx);
                                } else {
                                    c.finish_operation(false, format!("Échec (code {:?})", code), cx);
                                    c.append_stderr(format!(">>> Erreur (code {:?})", code), cx);
                                }
                            });
                            if success {
                                view.refresh_after_operation(cx);
                            }
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn upgrade_all(&mut self, cx: &mut Context<Self>) {
        self.console.update(cx, |c, cx| {
            c.clear_logs(cx);
            c.start_operation("Mise à niveau globale", cx);
            c.append_stdout(">>> Démarrage de la mise à niveau globale du système...", cx);
        });

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        self.client.upgrade_system(tx);
        let console_model = self.console.clone();

        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.recv().await {
                match event {
                    LogStreamEvent::Line(line) => {
                        let _ = console_model.update(cx, |c, cx| c.append_stdout(line, cx));
                    }
                    LogStreamEvent::ErrorLine(err) => {
                        let _ = console_model.update(cx, |c, cx| c.append_stderr(err, cx));
                    }
                    LogStreamEvent::Finished(success, code) => {
                        let _ = this.update(cx, |view, cx| {
                            view.console.update(cx, |c, cx| {
                                if success {
                                    c.finish_operation(true, "Système à jour", cx);
                                    c.append_stdout(">>> Mise à niveau terminée avec succès.", cx);
                                } else if code == Some(126) || code == Some(127) {
                                    c.finish_operation(false, "Authentification Polkit annulée", cx);
                                } else {
                                    c.finish_operation(false, format!("Erreur (code {:?})", code), cx);
                                }
                            });
                            if success {
                                view.catalog.update(cx, |cat, cx| cat.set_updates(Vec::new(), cx));
                                view.refresh_after_operation(cx);
                            }
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn refresh_after_operation(&mut self, cx: &mut Context<Self>) {
        self.catalog.update(cx, |cat, cx| cat.invalidate_cache(cx));
        self.load_initial_data(cx);
    }

    pub fn copy_logs_to_clipboard(&mut self, cx: &mut Context<Self>) {
        let full_text = self
            .console
            .read(cx)
            .logs
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        if !full_text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(full_text));
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

    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.console.update(cx, |c, cx| c.clear_logs(cx));
    }

    pub fn toggle_auto_scroll(&mut self, cx: &mut Context<Self>) {
        self.console.update(cx, |c, cx| c.toggle_auto_scroll(cx));
    }

    pub fn on_splitter_pointer_down(&mut self, _event: &MouseDownEvent, cx: &mut Context<Self>) {
        self.is_resizing_pane = true;
        cx.notify();
    }

    pub fn on_splitter_pointer_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if self.is_resizing_pane {
            let offset_x = event.position.x;
            let sidebar_w = if self.layout.read(cx).is_sidebar_collapsed {
                56.0
            } else {
                200.0
            };
            let new_w = (f32::from(offset_x) - sidebar_w).clamp(240.0, 800.0);
            self.layout.update(cx, |l, cx| {
                l.set_list_pane_width(new_w, cx);
            });
        }
    }

    pub fn on_splitter_pointer_up(&mut self, _event: &MouseUpEvent, cx: &mut Context<Self>) {
        if self.is_resizing_pane {
            self.is_resizing_pane = false;
            cx.notify();
        }
    }

    pub fn on_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;

        // Hotkey : Ctrl+Shift+D pour activer / désactiver le HUD de performances
        if modifiers.control && modifiers.shift && (key == "D" || key == "d") {
            self.layout.update(cx, |l, cx| l.toggle_diagnostics_hud(cx));
            return;
        }

        if modifiers.control || modifiers.alt || modifiers.platform {
            return;
        }

        let items = self.get_sorted_items(cx);
        let current_sel = self.catalog.read(cx).selected_package.clone();
        let current_idx = current_sel.and_then(|sel| items.iter().position(|p| p.name == sel.name));

        match key {
            "down" => {
                let next_idx = match current_idx {
                    Some(idx) if idx + 1 < items.len() => idx + 1,
                    None if !items.is_empty() => 0,
                    _ => return,
                };
                if let Some(pkg) = items.get(next_idx) {
                    self.select_package_item(pkg.clone(), cx);
                    self.scroll_handle
                        .scroll_to_item(next_idx, ScrollStrategy::Top);
                }
                return;
            }
            "up" => {
                let prev_idx = match current_idx {
                    Some(idx) if idx > 0 => idx - 1,
                    _ => return,
                };
                if let Some(pkg) = items.get(prev_idx) {
                    self.select_package_item(pkg.clone(), cx);
                    self.scroll_handle
                        .scroll_to_item(prev_idx, ScrollStrategy::Top);
                }
                return;
            }
            "backspace" => {
                let q = self.catalog.read(cx).search_query.clone();
                let mut chars = q.chars();
                chars.next_back();
                let new_q = chars.as_str().to_string();
                self.catalog.update(cx, |cat, cx| cat.set_search_query(new_q.clone(), cx));
            }
            "escape" => {
                self.catalog.update(cx, |cat, cx| cat.set_search_query(String::new(), cx));
                window.blur();
                cx.notify();
                return;
            }
            "enter" => {
                let q = self.catalog.read(cx).search_query.clone();
                self.search_generation = self.search_generation.wrapping_add(1);
                cx.notify();
                self.perform_search(q, cx);
                return;
            }
            k if k.len() == 1 => {
                let mut q = self.catalog.read(cx).search_query.clone();
                q.push_str(k);
                self.catalog.update(cx, |cat, cx| cat.set_search_query(q, cx));
            }
            _ => return,
        }

        cx.notify();

        // Debounce 50 ms pour recherche ultra-réactive
        self.search_generation = self.search_generation.wrapping_add(1);
        let generation = self.search_generation;
        let query = self.catalog.read(cx).search_query.clone();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(50))
                .await;

            let _ = this.update(cx, |view, cx| {
                if view.search_generation == generation {
                    view.perform_search(query, cx);
                }
            });
        })
        .detach();
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let route = self.layout.read(cx).active_route;
        let is_sidebar_collapsed = self.layout.read(cx).is_sidebar_collapsed;
        let view_mode = self.layout.read(cx).package_view_mode;
        let active_inspector_tab = self.layout.read(cx).active_inspector_tab;
        let list_pane_w = self.layout.read(cx).list_pane_width;
        let show_hud = self.layout.read(cx).show_diagnostics_hud;

        let (logs, op_status, log_open, auto_scroll, console_height) = {
            let c = self.console.read(cx);
            (c.logs.clone(), c.status.clone(), c.is_open, c.auto_scroll, c.height)
        };
        let is_busy = matches!(op_status, crate::components::log_drawer::OperationStatus::Running(_));

        let (search_query, selected_pkg, alpm_details, (total_cnt, off_cnt, aur_cnt, fp_cnt), active_source_filter, updates_count) = {
            let cat = self.catalog.read(cx);
            let sel = cat.selected_package.clone();
            let details = sel.as_ref().and_then(|p| cat.detail_cache.get(&p.name).cloned());
            (cat.search_query.clone(), sel, details, cat.counts(), cat.source_filter, cat.updates.len())
        };

        let items = self.get_sorted_items(cx);
        let active_rows = items.len();

        // Calcul des métriques pour le Performance HUD
        let now = Instant::now();
        let frame_time_ms = if let Some(last) = self.last_render {
            now.duration_since(last).as_secs_f32() * 1000.0
        } else {
            16.6
        };
        self.last_render = Some(now);
        let fps = if frame_time_ms > 0.0 {
            (1000.0 / frame_time_ms).clamp(1.0, 240.0)
        } else {
            60.0
        };
        self.layout.update(cx, |l, cx| {
            l.update_metrics(fps, frame_time_ms, active_rows, cx);
        });
        let memory_mb = DiagnosticsHud::read_rss_memory_mb();

        let entity = cx.entity().clone();

        // ── 1. Volet de navigation vertical (NavRail) ────────────────────────
        let entity_nav = entity.clone();
        let entity_side = entity.clone();
        let entity_hud = entity.clone();

        let nav_rail = NavRail::render(NavRailProps {
            active_route: route,
            is_collapsed: is_sidebar_collapsed,
            update_count: updates_count,
            theme: &theme,
            hud_active: show_hud,
            on_select_route: Rc::new(move |r, _window, cx| {
                entity_nav.update(cx, |view, cx| {
                    view.switch_route(r, cx);
                });
            }),
            on_toggle_sidebar: Rc::new(move |_ev, _window, cx| {
                entity_side.update(cx, |view, cx| {
                    view.layout.update(cx, |l, cx| l.toggle_sidebar(cx));
                });
            }),
            on_toggle_hud: Rc::new(move |_ev, _window, cx| {
                entity_hud.update(cx, |view, cx| {
                    view.layout.update(cx, |l, cx| l.toggle_diagnostics_hud(cx));
                });
            }),
        });

        // ── 2. Contenu principal selon la route ──────────────────────────────
        let main_content = match route {
            NavRoute::Search | NavRoute::Installed => {
                // ── Barre de recherche & Filtres multi-sources ──────────────
                let search_field = {
                    let has_text = !search_query.is_empty();
                    let is_focused = self.search_focus.is_focused(window);
                    let border_col = if is_focused {
                        theme.border_focus
                    } else {
                        theme.border
                    };

                    let entity_focus = entity.clone();
                    let entity_clear = entity.clone();

                    div()
                        .flex_1()
                        .max_w(px(520.0))
                        .h(px(36.0))
                        .flex()
                        .items_center()
                        .px_3()
                        .rounded_md()
                        .border_1()
                        .border_color(border_col)
                        .bg(theme.bg_surface)
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                            entity_focus.update(cx, |view, _cx| {
                                window.focus(&view.search_focus);
                            });
                        })
                        .child(
                            div()
                                .text_color(if is_focused {
                                    theme.accent
                                } else {
                                    theme.text_muted
                                })
                                .mr_2()
                                .child("🔍"),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(if has_text {
                                    theme.text_primary
                                } else {
                                    theme.text_muted
                                })
                                .child(if has_text {
                                    search_query.clone()
                                } else {
                                    "Rechercher un paquet (dépôts, AUR, Flatpak)...".to_string()
                                }),
                        )
                        .when(has_text, |d| {
                            d.child(
                                div()
                                    .cursor_pointer()
                                    .text_color(theme.text_muted)
                                    .hover(move |s| s.text_color(theme.text_primary))
                                    .child("✕")
                                    .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                                        entity_clear.update(cx, |view, cx| {
                                            view.catalog.update(cx, |cat, cx| {
                                                cat.set_search_query(String::new(), cx)
                                            });
                                            view.perform_search(String::new(), cx);
                                        });
                                    }),
                            )
                        })
                };

                // Filter Pills multi-sources
                let entity_filter = entity.clone();
                let filter_pills = FilterPills::render(FilterPillsProps {
                    active_filter: active_source_filter,
                    total_count: total_cnt,
                    official_count: off_cnt,
                    aur_count: aur_cnt,
                    flatpak_count: fp_cnt,
                    theme: &theme,
                    on_select_filter: Rc::new(move |f, _w, cx| {
                        entity_filter.update(cx, |view, cx| {
                            view.catalog.update(cx, |cat, cx| cat.set_source_filter(f, cx));
                        });
                    }),
                });

                // Bouton bascule de présentation (Cartes vs Tableau)
                let entity_vm = entity.clone();
                let view_mode_btn = div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.bg_surface)
                    .text_xs()
                    .cursor_pointer()
                    .text_color(theme.text_secondary)
                    .hover(move |s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
                    .child(if view_mode == PackageViewMode::Table {
                        "☷ Mode Cartes"
                    } else {
                        "☰ Mode Tableau"
                    })
                    .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                        entity_vm.update(cx, |view, cx| {
                            let next_mode = if view.layout.read(cx).package_view_mode == PackageViewMode::Table {
                                PackageViewMode::Cards
                            } else {
                                PackageViewMode::Table
                            };
                            view.layout.update(cx, |l, cx| l.set_view_mode(next_mode, cx));
                        });
                    });

                let toolbar = div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_2p5()
                    .bg(theme.bg_app)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(div().flex().items_center().gap_4().child(search_field).child(filter_pills))
                    .child(view_mode_btn);

                // ── Volet gauche (Liste de paquets) ──────────────────────────
                let entity_sel_table = entity.clone();
                let entity_sort = entity.clone();

                let list_content: AnyElement = if view_mode == PackageViewMode::Table {
                    PackageTable::render(PackageTableProps {
                        packages: &items,
                        selected_package: selected_pkg.as_ref(),
                        theme: &theme,
                        scroll_handle: self.table_scroll_handle.clone(),
                        sort_column: self.sort_column,
                        sort_direction: self.sort_direction,
                        on_select_package: Rc::new(move |pkg, _w, cx| {
                            let p = pkg.clone();
                            entity_sel_table.update(cx, |view, cx| {
                                view.select_package_item(p, cx);
                            });
                        }),
                        on_change_sort: Rc::new(move |col, _w, cx| {
                            entity_sort.update(cx, |view, cx| {
                                view.change_sort(col, cx);
                            });
                        }),
                    })
                    .into_any_element()
                } else {
                    let scroll_handle = self.scroll_handle.clone();
                    let package_count = items.len();
                    let card_items = items.clone();
                    let card_sel = selected_pkg.clone();
                    let card_entity = entity.clone();

                    uniform_list("package_cards_list", package_count, move |range, _window, _cx| {
                        let mut elements = Vec::with_capacity(range.end - range.start);
                        for idx in range {
                            if let Some(pkg) = card_items.get(idx) {
                                let is_sel = card_sel.as_ref().map(|s| s.name == pkg.name).unwrap_or(false);
                                let pkg_clone = pkg.clone();
                                let ent = card_entity.clone();

                                let el = div()
                                    .h(px(78.0))
                                    .px_2()
                                    .pb_1p5()
                                    .child(PackageCard::render(PackageCardProps {
                                        package: pkg,
                                        is_selected: is_sel,
                                        theme: &theme,
                                    }))
                                    .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                                        let p = pkg_clone.clone();
                                        ent.update(cx, |view, cx| {
                                            view.select_package_item(p, cx);
                                        });
                                    });
                                elements.push(el);
                            }
                        }
                        elements
                    })
                    .size_full()
                    .track_scroll(scroll_handle)
                    .into_any_element()
                };

                let left_pane = div()
                    .w(px(list_pane_w))
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .bg(theme.bg_surface)
                    .child(list_content);

                // ── Barre de redimensionnement (Splitter) ────────────────────
                let is_resizing = self.is_resizing_pane;
                let entity_splitter = entity.clone();
                let splitter = div()
                    .w(px(6.0))
                    .h_full()
                    .bg(if is_resizing {
                        theme.accent
                    } else {
                        theme.border
                    })
                    .cursor_col_resize()
                    .hover(move |s| s.bg(theme.accent_hover))
                    .on_mouse_down(MouseButton::Left, move |e, _w, cx| {
                        entity_splitter.update(cx, |view, cx| {
                            view.on_splitter_pointer_down(e, cx);
                        });
                    });

                // ── Volet droit (Inspector avec onglets) ──────────────────────
                let entity_inst = entity.clone();
                let entity_rem = entity.clone();
                let entity_tab = entity.clone();
                let entity_nav_dep = entity.clone();

                let right_pane = div()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .bg(theme.bg_app)
                    .child(PackageDetailsView::render(PackageDetailsProps {
                        package: selected_pkg.as_ref(),
                        alpm_details: alpm_details.as_ref(),
                        theme: &theme,
                        is_busy,
                        active_tab: active_inspector_tab,
                        file_list: &self.file_list,
                        on_install: Some(Rc::new(move |_e, _w, cx| {
                            entity_inst.update(cx, |view, cx| view.install_selected(cx));
                        })),
                        on_remove: Some(Rc::new(move |_e, _w, cx| {
                            entity_rem.update(cx, |view, cx| view.remove_selected(cx));
                        })),
                        on_change_tab: Some(Rc::new(move |t, _w, cx| {
                            entity_tab.update(cx, |view, cx| {
                                view.layout.update(cx, |l, cx| l.set_inspector_tab(t, cx));
                            });
                        })),
                        on_navigate_dep: Some(Rc::new(move |dep_name, _w, cx| {
                            entity_nav_dep.update(cx, |view, cx| {
                                view.perform_search(dep_name, cx);
                            });
                        })),
                    }));

                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .h_full()
                    .overflow_hidden()
                    .child(toolbar)
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .child(left_pane)
                            .child(splitter)
                            .child(right_pane),
                    )
            }
            NavRoute::Updates => {
                let entity_up = entity.clone();
                let updates_list = self.catalog.read(cx).updates.clone();

                let content = div()
                    .id("updates_scroll_view")
                    .flex()
                    .flex_col()
                    .size_full()
                    .p_6()
                    .overflow_scroll()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .mb_6()
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text_primary)
                                    .child(format!("Mises à jour système ({})", updates_list.len())),
                            )
                            .when(!updates_list.is_empty(), |d| {
                                d.child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .rounded_md()
                                        .bg(theme.accent)
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme.bg_app)
                                        .cursor_pointer()
                                        .hover(move |s| s.bg(theme.accent_hover))
                                        .child("Tout mettre à jour")
                                        .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                                            entity_up.update(cx, |view, cx| view.upgrade_all(cx));
                                        }),
                                )
                            }),
                    );

                div().flex_1().h_full().child(content)
            }
            NavRoute::Settings => {
                let settings = SettingsView::render(SettingsViewProps {
                    shelly_settings: &self.shelly_settings,
                    gpui_config: &self.gpui_config,
                    theme: &theme,
                });

                let news = NewsView::render(NewsViewProps {
                    news: &self.news,
                    is_loading: self.is_loading_news,
                    theme: &theme,
                });

                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .overflow_hidden()
                    .child(div().id("settings_pane").w_1_2().h_full().overflow_scroll().child(settings))
                    .child(div().id("news_pane").w_1_2().h_full().overflow_scroll().child(news))
            }
        };

        // ── 3. Tiroir de logs / console d'opérations ─────────────────────────
        let entity_log_toggle = entity.clone();
        let entity_log_copy = entity.clone();
        let entity_log_clear = entity.clone();
        let entity_log_auto = entity.clone();
        let copied_feedback = self.logs_copied_feedback;

        let log_drawer = LogDrawer::render(LogDrawerProps {
            logs: &logs,
            status: &op_status,
            is_open: log_open,
            auto_scroll: auto_scroll,
            height: console_height,
            copied_feedback,
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
                entity_log_clear.update(cx, |view, cx| view.clear_logs(cx));
            })),
            on_toggle_autoscroll: Some(Rc::new(move |_e, _w, cx| {
                entity_log_auto.update(cx, |view, cx| view.toggle_auto_scroll(cx));
            })),
        });

        // ── 4. HUD de diagnostic de performance flottant ──────────────────────
        let entity_hud_close = entity.clone();
        let diagnostics_hud = if show_hud {
            let layout_read = self.layout.read(cx);
            Some(
                div()
                    .absolute()
                    .bottom(px(40.0))
                    .right(px(24.0))
                    .child(DiagnosticsHud::render(DiagnosticsHudProps {
                        fps: layout_read.fps,
                        frame_time_ms: layout_read.frame_time_ms,
                        active_rows: layout_read.active_rows,
                        memory_mb,
                        theme: &theme,
                        on_close: Some(Rc::new(move |_e, _w, cx| {
                            entity_hud_close.update(cx, |view, cx| {
                                view.layout.update(cx, |l, cx| l.toggle_diagnostics_hud(cx));
                            });
                        })),
                    })),
            )
        } else {
            None
        };

        // ── 5. Assemblage final de la fenêtre ────────────────────────────────
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
            .child(nav_rail)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(div().flex_1().h_full().overflow_hidden().child(main_content))
                    .child(log_drawer),
            )
            .children(diagnostics_hud)
    }
}
