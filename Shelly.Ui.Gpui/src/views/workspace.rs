use crate::backend::client::ShellyClient;
use crate::backend::models::{AlpmPackage, ArchNewsItem, UnifiedPackage};
use crate::backend::process::LogStreamEvent;
use crate::components::log_drawer::{LogDrawer, LogDrawerProps, LogEntry, OperationStatus};
use crate::components::package_card::{PackageCard, PackageCardProps};
use crate::components::status_pill::StatusPill;
use crate::config::{ConfigManager, GpuiUiConfig, ShellySettings};
use crate::theme::Theme;
use crate::views::details::{PackageDetailsProps, PackageDetailsView};
use crate::views::news::{NewsView, NewsViewProps};
use crate::views::settings::{SettingsView, SettingsViewProps};
use gpui::prelude::FluentBuilder;
use gpui::{uniform_list, ScrollStrategy, UniformListScrollHandle};
use gpui::*;
use std::collections::HashMap;
use std::rc::Rc;
use tokio::sync::mpsc;

pub const TAB_ALPM: usize = 0;
pub const TAB_AUR: usize = 1;
pub const TAB_FLATPAK: usize = 2;
pub const TAB_APPIMAGE: usize = 3;
pub const TAB_UPDATES: usize = 4;
pub const TAB_NEWS: usize = 5;
pub const TAB_SETTINGS: usize = 6;

pub struct WorkspaceView {
    pub client: ShellyClient,
    pub shelly_settings: ShellySettings,
    pub gpui_config: GpuiUiConfig,
    pub theme: Theme,
    pub active_tab: usize,
    pub search_query: String,
    /// FocusHandle qui rend la barre de recherche réceptive aux touches clavier
    pub search_focus: FocusHandle,
    /// Compteur de génération pour le debounce : chaque frappe l'incrémente ;
    /// la tâche async ne lance la recherche que si la génération correspond encore.
    pub search_generation: u64,
    pub packages: Vec<UnifiedPackage>,
    pub selected_index: Option<usize>,
    /// Détails complets ALPM chargés via `get_package_details` pour enrichir le volet droit
    pub selected_alpm_details: Option<AlpmPackage>,
    pub news: Vec<ArchNewsItem>,
    pub is_searching: bool,
    pub is_loading_news: bool,
    pub operation_logs: Vec<LogEntry>,
    pub operation_status: OperationStatus,
    pub log_drawer_open: bool,
    pub auto_scroll_logs: bool,
    pub logs_copied_feedback: bool,
    pub updates_count: usize,
    /// Largeur en pixels du volet de gauche (liste des paquets), ajustable à la souris
    pub list_pane_width: f32,
    /// Indicateur actif pendant le glisser-déposer de redimensionnement
    pub is_resizing_pane: bool,
    /// Handle de défilement pour la liste virtualisée
    pub scroll_handle: UniformListScrollHandle,
    /// Cache en mémoire pour éviter les requêtes subprocess redondantes lors de l'inspection des paquets
    pub package_detail_cache: HashMap<String, AlpmPackage>,
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

        let active_tab = gpui_config.last_selected_tab;
        let log_drawer_open = gpui_config.log_drawer_open;

        let mut view = Self {
            client: ShellyClient::default(),
            shelly_settings,
            gpui_config,
            theme,
            active_tab,
            search_query: String::new(),
            search_focus: cx.focus_handle(),
            search_generation: 0,
            packages: Vec::new(),
            selected_index: None,
            selected_alpm_details: None,
            news: Vec::new(),
            is_searching: false,
            is_loading_news: false,
            operation_logs: Vec::new(),
            operation_status: OperationStatus::Idle,
            log_drawer_open,
            auto_scroll_logs: true,
            logs_copied_feedback: false,
            updates_count: 0,
            list_pane_width: 440.0,
            is_resizing_pane: false,
            scroll_handle: UniformListScrollHandle::new(),
            package_detail_cache: HashMap::new(),
        };

        view.load_initial_data(cx);
        view
    }

    pub fn load_initial_data(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(updates) = client.list_updates().await {
                let count = updates.len();
                let _ = this.update(cx, |view, cx| {
                    view.updates_count = count;
                    cx.notify();
                });
            }

            if let Ok(installed) = client.search_installed("").await {
                let unified: Vec<UnifiedPackage> = installed
                    .into_iter()
                    .take(60)
                    .map(|p| UnifiedPackage::from_alpm(p, true))
                    .collect();

                let _ = this.update(cx, |view, cx| {
                    view.packages = unified;
                    if !view.packages.is_empty() {
                        view.selected_index = Some(0);
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub fn switch_tab(&mut self, tab: usize, cx: &mut Context<Self>) {
        self.active_tab = tab;
        self.selected_index = None;
        self.selected_alpm_details = None;
        self.gpui_config.last_selected_tab = tab;
        let _ = ConfigManager::save_gpui_config(&self.gpui_config);

        match tab {
            TAB_ALPM => self.perform_search(self.search_query.clone(), cx),
            TAB_AUR => self.search_aur_tab(self.search_query.clone(), cx),
            TAB_FLATPAK => self.search_flatpak_tab(self.search_query.clone(), cx),
            TAB_APPIMAGE => self.load_appimages(cx),
            TAB_UPDATES => self.load_updates(cx),
            TAB_NEWS => self.load_news(cx),
            TAB_SETTINGS => cx.notify(),
            _ => cx.notify(),
        }
    }

    /// Sélectionne un paquet et lance le chargement de ses détails ALPM complets en arrière-plan
    pub fn select_package(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.selected_index = Some(idx);
        self.selected_alpm_details = None;
        cx.notify();

        if let Some(pkg) = self.packages.get(idx) {
            if pkg.source_type == "ALPM" {
                let name = pkg.name.clone();

                // 1. Vérifier d'abord le cache de présentation en mémoire
                if let Some(cached) = self.package_detail_cache.get(&name) {
                    self.selected_alpm_details = Some(cached.clone());
                    cx.notify();
                    return;
                }

                let client = self.client.clone();
                cx.spawn(async move |this, cx| {
                    if let Ok(Some(details)) = client.get_package_details(&name).await {
                        let _ = this.update(cx, |view, cx| {
                            view.package_detail_cache.insert(name.clone(), details.clone());
                            if view.selected_index == Some(idx) {
                                view.selected_alpm_details = Some(details);
                                cx.notify();
                            }
                        });
                    }
                })
                .detach();
            }
        }
    }

    pub fn perform_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_query = query.clone();
        self.is_searching = true;
        self.selected_alpm_details = None;
        cx.notify();

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let pkgs = if query.trim().is_empty() {
                client.search_installed("").await.unwrap_or_default()
            } else {
                client.search_standard(&query).await.unwrap_or_default()
            };

            let unified: Vec<UnifiedPackage> = pkgs
                .into_iter()
                .map(|p| {
                    let is_installed = p.install_date.is_some()
                        || p.install_reason.as_deref() != Some("Not Installed");
                    UnifiedPackage::from_alpm(p, is_installed)
                })
                .collect();

            let _ = this.update(cx, |view, cx| {
                view.packages = unified;
                view.is_searching = false;
                if !view.packages.is_empty() {
                    view.selected_index = Some(0);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn search_aur_tab(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_query = query.clone();
        self.is_searching = true;
        self.selected_alpm_details = None;
        cx.notify();

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let q = if query.trim().is_empty() { "git" } else { &query };
            let results = client.search_aur(q).await.unwrap_or_default();
            let unified: Vec<UnifiedPackage> = results
                .into_iter()
                .map(|p| UnifiedPackage::from_aur(p, false))
                .collect();

            let _ = this.update(cx, |view, cx| {
                view.packages = unified;
                view.is_searching = false;
                if !view.packages.is_empty() {
                    view.selected_index = Some(0);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn search_flatpak_tab(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_query = query.clone();
        self.is_searching = true;
        self.selected_alpm_details = None;
        cx.notify();

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let q = if query.trim().is_empty() { "browser" } else { &query };
            let hits = client.search_flatpak(q).await.unwrap_or_default();
            let unified: Vec<UnifiedPackage> = hits
                .into_iter()
                .map(|h| UnifiedPackage::from_flatpak(h, false))
                .collect();

            let _ = this.update(cx, |view, cx| {
                view.packages = unified;
                view.is_searching = false;
                if !view.packages.is_empty() {
                    view.selected_index = Some(0);
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Charge la liste des AppImages gérées par Shelly via le CLI (`shelly list appimage -j`)
    pub fn load_appimages(&mut self, cx: &mut Context<Self>) {
        self.is_searching = true;
        self.selected_alpm_details = None;
        cx.notify();

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let appimages = client.list_appimages().await.unwrap_or_default();
            let unified: Vec<UnifiedPackage> = appimages
                .into_iter()
                .map(UnifiedPackage::from_appimage)
                .collect();

            let _ = this.update(cx, |view, cx| {
                view.packages = unified;
                view.is_searching = false;
                if !view.packages.is_empty() {
                    view.selected_index = Some(0);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn load_updates(&mut self, cx: &mut Context<Self>) {
        self.is_searching = true;
        self.selected_alpm_details = None;
        cx.notify();

        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let updates = client.list_updates().await.unwrap_or_default();
            let count = updates.len();

            let unified: Vec<UnifiedPackage> = updates
                .into_iter()
                .map(|u| UnifiedPackage {
                    name: u.name,
                    version: u.old_version,
                    description: format!("Mise à jour disponible vers {}", u.new_version),
                    source_type: u.package_type.unwrap_or_else(|| "ALPM".to_string()),
                    repository_or_remote: u.repository.unwrap_or_else(|| "repos".to_string()),
                    is_installed: true,
                    has_update: true,
                    new_version: Some(u.new_version),
                    inner: crate::backend::models::UnifiedPackageSource::Standard(Default::default()),
                })
                .collect();

            let _ = this.update(cx, |view, cx| {
                view.packages = unified;
                view.updates_count = count;
                view.is_searching = false;
                if !view.packages.is_empty() {
                    view.selected_index = Some(0);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn load_news(&mut self, cx: &mut Context<Self>) {
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

    pub fn toggle_log_drawer(&mut self, cx: &mut Context<Self>) {
        self.log_drawer_open = !self.log_drawer_open;
        self.gpui_config.log_drawer_open = self.log_drawer_open;
        let _ = ConfigManager::save_gpui_config(&self.gpui_config);
        cx.notify();
    }

    pub fn copy_logs_to_clipboard(&mut self, cx: &mut Context<Self>) {
        let text = self
            .operation_logs
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.logs_copied_feedback = true;
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(2000))
                .await;
            let _ = this.update(cx, |view, cx| {
                view.logs_copied_feedback = false;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.operation_logs.clear();
        cx.notify();
    }

    pub fn install_selected(&mut self, cx: &mut Context<Self>) {
        let Some(index) = self.selected_index else {
            return;
        };
        let Some(pkg) = self.packages.get(index) else {
            return;
        };

        let name = pkg.name.clone();
        let is_aur = pkg.source_type == "AUR";
        let is_flatpak = pkg.source_type == "Flatpak";

        self.operation_logs.clear();
        self.operation_logs
            .push(LogEntry::stdout(format!(">>> Lancement de l'installation de {}...", name)));
        self.operation_status =
            OperationStatus::Running(format!("Installation de {}", name));
        self.log_drawer_open = true;
        cx.notify();

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        self.client.install_package(&name, is_aur, is_flatpak, tx);

        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.recv().await {
                match event {
                    LogStreamEvent::Line(line) => {
                        let _ = this.update(cx, |view, cx| {
                            view.operation_logs.push(LogEntry::stdout(line));
                            cx.notify();
                        });
                    }
                    LogStreamEvent::ErrorLine(err) => {
                        let _ = this.update(cx, |view, cx| {
                            view.operation_logs.push(LogEntry::stderr(err));
                            cx.notify();
                        });
                    }
                    LogStreamEvent::Finished(success, code) => {
                        let _ = this.update(cx, |view, cx| {
                            if success {
                                view.operation_status = OperationStatus::Success(
                                    format!("{} installé avec succès", name),
                                );
                                view.operation_logs
                                    .push(LogEntry::stdout(">>> Opération terminée avec succès."));
                                view.refresh_after_operation(cx);
                            } else {
                                view.log_drawer_open = true;
                                if code == Some(126) || code == Some(127) {
                                    view.operation_status = OperationStatus::Error(
                                        "Authentification Polkit annulée".to_string(),
                                    );
                                    view.operation_logs.push(LogEntry::stderr(
                                        ">>> Opération annulée : invite d'authentification Polkit fermée ou refusée."
                                    ));
                                } else {
                                    let last_err = view.operation_logs.iter().rev().find(|l| l.is_stderr).map(|l| l.text.clone());
                                    let err_desc = last_err.unwrap_or_else(|| format!("Échec (code {:?})", code));
                                    view.operation_status = OperationStatus::Error(err_desc);
                                    view.operation_logs.push(LogEntry::stderr(format!(
                                        ">>> Échec de l'opération (code {:?})",
                                        code
                                    )));
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn remove_selected(&mut self, cx: &mut Context<Self>) {
        let Some(index) = self.selected_index else {
            return;
        };
        let Some(pkg) = self.packages.get(index) else {
            return;
        };

        let name = pkg.name.clone();
        let is_flatpak = pkg.source_type == "Flatpak";

        self.operation_logs.clear();
        self.operation_logs
            .push(LogEntry::stdout(format!(">>> Suppression du paquet {}...", name)));
        self.operation_status =
            OperationStatus::Running(format!("Suppression de {}", name));
        self.log_drawer_open = true;
        cx.notify();

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        self.client.remove_package(&name, is_flatpak, tx);

        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.recv().await {
                match event {
                    LogStreamEvent::Line(line) => {
                        let _ = this.update(cx, |view, cx| {
                            view.operation_logs.push(LogEntry::stdout(line));
                            cx.notify();
                        });
                    }
                    LogStreamEvent::ErrorLine(err) => {
                        let _ = this.update(cx, |view, cx| {
                            view.operation_logs.push(LogEntry::stderr(err));
                            cx.notify();
                        });
                    }
                    LogStreamEvent::Finished(success, code) => {
                        let _ = this.update(cx, |view, cx| {
                            if success {
                                view.operation_status =
                                    OperationStatus::Success(format!("{} désinstallé", name));
                                view.operation_logs
                                    .push(LogEntry::stdout(">>> Désinstallation terminée avec succès."));
                                view.refresh_after_operation(cx);
                            } else {
                                view.log_drawer_open = true;
                                if code == Some(126) || code == Some(127) {
                                    view.operation_status = OperationStatus::Error(
                                        "Authentification Polkit annulée".to_string(),
                                    );
                                    view.operation_logs.push(LogEntry::stderr(
                                        ">>> Opération annulée : invite d'authentification Polkit fermée ou refusée."
                                    ));
                                } else {
                                    let last_err = view.operation_logs.iter().rev().find(|l| l.is_stderr).map(|l| l.text.clone());
                                    let err_desc = last_err.unwrap_or_else(|| format!("Échec (code {:?})", code));
                                    view.operation_status = OperationStatus::Error(err_desc);
                                    view.operation_logs.push(LogEntry::stderr(format!(
                                        ">>> Échec de la suppression (code {:?})",
                                        code
                                    )));
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    pub fn upgrade_all(&mut self, cx: &mut Context<Self>) {
        self.operation_logs.clear();
        self.operation_logs
            .push(LogEntry::stdout(">>> Démarrage de la mise à niveau globale du système...".to_string()));
        self.operation_status = OperationStatus::Running("Mise à niveau globale".to_string());
        self.log_drawer_open = true;
        cx.notify();

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        self.client.upgrade_system(tx);

        cx.spawn(async move |this, cx| {
            while let Some(event) = rx.recv().await {
                match event {
                    LogStreamEvent::Line(line) => {
                        let _ = this.update(cx, |view, cx| {
                            view.operation_logs.push(LogEntry::stdout(line));
                            cx.notify();
                        });
                    }
                    LogStreamEvent::ErrorLine(err) => {
                        let _ = this.update(cx, |view, cx| {
                            view.operation_logs.push(LogEntry::stderr(err));
                            cx.notify();
                        });
                    }
                    LogStreamEvent::Finished(success, code) => {
                        let _ = this.update(cx, |view, cx| {
                            if success {
                                view.operation_status =
                                    OperationStatus::Success("Système à jour".to_string());
                                view.operation_logs
                                    .push(LogEntry::stdout(">>> Mise à niveau terminée avec succès."));
                                view.updates_count = 0;
                                view.refresh_after_operation(cx);
                            } else {
                                view.log_drawer_open = true;
                                if code == Some(126) || code == Some(127) {
                                    view.operation_status = OperationStatus::Error(
                                        "Authentification Polkit annulée".to_string(),
                                    );
                                    view.operation_logs.push(LogEntry::stderr(
                                        ">>> Opération annulée : invite d'authentification Polkit fermée ou refusée."
                                    ));
                                } else {
                                    let last_err = view.operation_logs.iter().rev().find(|l| l.is_stderr).map(|l| l.text.clone());
                                    let err_desc = last_err.unwrap_or_else(|| format!("Erreur (code {:?})", code));
                                    view.operation_status = OperationStatus::Error(err_desc);
                                    view.operation_logs.push(LogEntry::stderr(format!(
                                        ">>> Erreur lors de la mise à niveau (code {:?})",
                                        code
                                    )));
                                }
                            }
                            cx.notify();
                        });
                        break;
                    }
                }
            }
        })
        .detach();
    }

    /// Rafraîchit les paquets et le compteur de mises à jour après une opération réussie
    pub fn refresh_after_operation(&mut self, cx: &mut Context<Self>) {
        self.package_detail_cache.clear();
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            if let Ok(updates) = client.list_updates().await {
                let count = updates.len();
                let _ = this.update(cx, |view, cx| {
                    view.updates_count = count;
                    cx.notify();
                });
            }
        })
        .detach();

        match self.active_tab {
            TAB_ALPM => self.perform_search(self.search_query.clone(), cx),
            TAB_AUR => self.search_aur_tab(self.search_query.clone(), cx),
            TAB_FLATPAK => self.search_flatpak_tab(self.search_query.clone(), cx),
            TAB_APPIMAGE => self.load_appimages(cx),
            TAB_UPDATES => self.load_updates(cx),
            TAB_NEWS => self.load_news(cx),
            _ => {}
        }
    }

    /// Traite une frappe clavier dans la barre de recherche.
    ///
    /// - Caractère imprimable (len == 1, sans Ctrl/Alt/Platform) → ajouté à la requête
    /// - `backspace` → supprime le dernier caractère Unicode
    /// - `escape`    → vide la requête et relâche le focus
    /// - `enter`     → recherche immédiate, bypass debounce
    ///
    /// Debounce 300 ms via compteur de génération : la tâche async ne
    /// déclenche la recherche que si `search_generation` n'a pas changé.
    pub fn handle_search_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;

        // Navigation clavier instantanée (Flèches Haut/Bas pour inspecter les paquets)
        if key == "down" || key == "arrowdown" {
            if let Some(idx) = self.selected_index {
                if idx + 1 < self.packages.len() {
                    let next = idx + 1;
                    self.select_package(next, cx);
                    self.scroll_handle.scroll_to_item(next, ScrollStrategy::Top);
                }
            } else if !self.packages.is_empty() {
                self.select_package(0, cx);
                self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
            }
            return;
        }
        if key == "up" || key == "arrowup" {
            if let Some(idx) = self.selected_index {
                if idx > 0 {
                    let prev = idx - 1;
                    self.select_package(prev, cx);
                    self.scroll_handle.scroll_to_item(prev, ScrollStrategy::Top);
                }
            } else if !self.packages.is_empty() {
                self.select_package(0, cx);
                self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
            }
            return;
        }

        // Ignorer les combinaisons système (Ctrl, Alt, Platform/Super)
        if modifiers.control || modifiers.alt || modifiers.platform {
            return;
        }

        match key {
            "backspace" => {
                // Supprime le dernier graphème Unicode
                let mut chars = self.search_query.chars();
                chars.next_back();
                self.search_query = chars.as_str().to_string();
            }
            "escape" => {
                self.search_query.clear();
                window.blur();
                cx.notify();
                return;
            }
            "enter" => {
                // Recherche immédiate — bypass debounce
                let query = self.search_query.clone();
                let tab = self.active_tab;
                self.search_generation = self.search_generation.wrapping_add(1);
                cx.notify();
                self.fire_search(query, tab, cx);
                return;
            }
            k if k.len() == 1 => {
                self.search_query.push_str(k);
            }
            _ => return,
        }

        cx.notify();

        // ── Debounce 50 ms (Ultra-réactif / Buttery-smooth) ───────────────────
        self.search_generation = self.search_generation.wrapping_add(1);
        let generation = self.search_generation;
        let query = self.search_query.clone();
        let tab = self.active_tab;

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(std::time::Duration::from_millis(50)).await;

            let _ = this.update(cx, |view, cx| {
                if view.search_generation == generation {
                    view.fire_search(query, tab, cx);
                }
            });
        })
        .detach();
    }

    /// Déclenche la recherche pour l'onglet actif (ALPM / AUR / Flatpak uniquement).
    /// AppImages, Updates, News et Settings n'ont pas de recherche textuelle live.
    fn fire_search(&mut self, query: String, tab: usize, cx: &mut Context<Self>) {
        match tab {
            TAB_ALPM => self.perform_search(query, cx),
            TAB_AUR => self.search_aur_tab(query, cx),
            TAB_FLATPAK => self.search_flatpak_tab(query, cx),
            _ => {}
        }
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let active_tab = self.active_tab;
        let updates_count = self.updates_count;
        let is_busy = matches!(self.operation_status, OperationStatus::Running(_));

        let selected_pkg = self.selected_index.and_then(|idx| self.packages.get(idx));

        // ── Barre de navigation ──────────────────────────────────────────────
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_2p5()
            .bg(theme.bg_sidebar)
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.accent)
                            .child("⚡ Shelly GPUI"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(Self::tab_button(
                                "Dépôts ALPM",
                                active_tab == TAB_ALPM,
                                &theme,
                                cx.listener(|this, _, _, cx| this.switch_tab(TAB_ALPM, cx)),
                            ))
                            .child(Self::tab_button(
                                "AUR",
                                active_tab == TAB_AUR,
                                &theme,
                                cx.listener(|this, _, _, cx| this.switch_tab(TAB_AUR, cx)),
                            ))
                            .child(Self::tab_button(
                                "Flatpaks",
                                active_tab == TAB_FLATPAK,
                                &theme,
                                cx.listener(|this, _, _, cx| this.switch_tab(TAB_FLATPAK, cx)),
                            ))
                            .child(Self::tab_button(
                                "AppImages",
                                active_tab == TAB_APPIMAGE,
                                &theme,
                                cx.listener(|this, _, _, cx| this.switch_tab(TAB_APPIMAGE, cx)),
                            ))
                            .child(Self::tab_button_with_badge(
                                "Mises à jour",
                                active_tab == TAB_UPDATES,
                                updates_count,
                                &theme,
                                cx.listener(|this, _, _, cx| this.switch_tab(TAB_UPDATES, cx)),
                            ))
                            .child(Self::tab_button(
                                "Actualités",
                                active_tab == TAB_NEWS,
                                &theme,
                                cx.listener(|this, _, _, cx| this.switch_tab(TAB_NEWS, cx)),
                            ))
                            .child(Self::tab_button(
                                "Paramètres",
                                active_tab == TAB_SETTINGS,
                                &theme,
                                cx.listener(|this, _, _, cx| this.switch_tab(TAB_SETTINGS, cx)),
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            // Grisé si une opération est en cours
                            .bg(if is_busy { theme.border } else { theme.accent })
                            .hover(|s| s.bg(if is_busy { theme.border } else { theme.accent_hover }))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.bg_app)
                            .cursor_pointer()
                            .child(if updates_count > 0 {
                                format!("Tout mettre à jour ({})", updates_count)
                            } else {
                                "Tout mettre à jour".to_string()
                            })
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                                if !matches!(this.operation_status, OperationStatus::Running(_)) {
                                    this.upgrade_all(cx);
                                }
                            })),
                    ),
            );

        // ── Corps central ────────────────────────────────────────────────────
        let content = if active_tab == TAB_NEWS {
            div().size_full().child(NewsView::render(NewsViewProps {
                news: &self.news,
                theme: &theme,
                is_loading: self.is_loading_news,
            }))
        } else if active_tab == TAB_SETTINGS {
            div().size_full().child(SettingsView::render(SettingsViewProps {
                shelly_settings: &self.shelly_settings,
                gpui_config: &self.gpui_config,
                theme: &theme,
            }))
        } else {
            // Split view : liste à gauche, détails à droite
            let mut list_pane = div()
                .flex()
                .flex_col()
                .w(px(self.list_pane_width))
                .h_full()
                .bg(theme.bg_app);

            // ── Barre de recherche interactive ──────────────────────────────
            // Clique → focus → touches → debounce 300 ms → recherche live
            let search_focus = self.search_focus.clone();
            let is_search_focused = self.search_focus.is_focused(window);
            let search_query = self.search_query.clone();
            let is_searching = self.is_searching;

            // Bordure colorée si la barre est active
            let border_color = if is_search_focused {
                theme.border_focus
            } else {
                theme.border
            };

            // Texte affiché : requête en cours, ou placeholder si vide
            let display_text = if search_query.is_empty() {
                if is_search_focused {
                    "⌨  Tapez votre recherche...".to_string()
                } else {
                    "Rechercher  (cliquez ou tapez)".to_string()
                }
            } else if is_search_focused {
                // Curseur visuel en fin de saisie
                format!("{}▌", search_query)
            } else {
                search_query.clone()
            };

            let text_color = if search_query.is_empty() {
                theme.text_muted
            } else {
                theme.text_primary
            };

            let search_bar = div()
                .id("search_bar")
                .track_focus(&search_focus)
                .flex()
                .items_center()
                .justify_between()
                .p_3()
                .border_b_1()
                .border_color(border_color)
                .bg(if is_search_focused { theme.bg_surface_active } else { theme.bg_surface })
                // Clic → focus clavier
                .on_mouse_down(MouseButton::Left, {
                    let sf = search_focus.clone();
                    move |_, window, _cx| {
                        window.focus(&sf);
                    }
                })
                // Capture des touches clavier
                .on_key_down(cx.listener(|this, event, window, cx| {
                    this.handle_search_key(event, window, cx);
                }))
                .child(
                    div()
                        .flex_grow()
                        .text_xs()
                        .text_color(text_color)
                        .child(display_text),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_search_focused { theme.accent } else { theme.text_muted })
                        .child(if is_searching {
                            "Recherche..."
                        } else if is_search_focused {
                            "Esc pour annuler"
                        } else {
                            "Clic pour saisir"
                        }),
                );

            list_pane = list_pane.child(search_bar);

            let scroll_list = if self.is_searching {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(100.0))
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("Recherche des paquets en cours...")
                    .into_any_element()
            } else if self.packages.is_empty() {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(100.0))
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("Aucun paquet correspondant trouvé.")
                    .into_any_element()
            } else {
                let entity = cx.entity().clone();
                let selected_index = self.selected_index;
                let package_count = self.packages.len();
                let scroll_handle = self.scroll_handle.clone();

                uniform_list(
                    "packages_uniform_list",
                    package_count,
                    cx.processor(move |this, range: std::ops::Range<usize>, _window, _cx| {
                        let theme = this.theme;
                        let mut items = Vec::with_capacity(range.end - range.start);
                        for idx in range {
                            if let Some(pkg) = this.packages.get(idx) {
                                let is_selected = selected_index == Some(idx);
                                let card_entity = entity.clone();
                                let item = div()
                                    .id(idx)
                                    .h(px(78.0))
                                    .px_2()
                                    .pb_1p5()
                                    .child(PackageCard::render(PackageCardProps {
                                        package: pkg,
                                        is_selected,
                                        theme: &theme,
                                    }))
                                    .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                                        card_entity.update(cx, |view, cx| {
                                            view.select_package(idx, cx);
                                        });
                                    });
                                items.push(item);
                            }
                        }
                        items
                    }),
                )
                .size_full()
                .track_scroll(scroll_handle)
                .into_any_element()
            };

            list_pane = list_pane.child(
                div()
                    .id("packages_scroll_container")
                    .flex_grow()
                    .h_full()
                    .overflow_hidden()
                    .child(scroll_list),
            );

            let details_pane = div()
                .flex_grow()
                .h_full()
                .child(PackageDetailsView::render(PackageDetailsProps {
                    package: selected_pkg,
                    alpm_details: self.selected_alpm_details.as_ref(),
                    theme: &theme,
                    is_busy,
                    on_install: Some(Rc::new(cx.listener(|this, _, _, cx| {
                        this.install_selected(cx);
                    }))),
                    on_remove: Some(Rc::new(cx.listener(|this, _, _, cx| {
                        this.remove_selected(cx);
                    }))),
                }));

            let is_resizing = self.is_resizing_pane;
            let splitter = div()
                .id("pane_splitter")
                .w(px(6.0))
                .h_full()
                .bg(if is_resizing { theme.accent } else { theme.border })
                .cursor_col_resize()
                .hover(|s| s.bg(theme.accent_hover))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    this.is_resizing_pane = true;
                    cx.notify();
                }));

            div()
                .flex()
                .size_full()
                .on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
                    if this.is_resizing_pane {
                        this.is_resizing_pane = false;
                        cx.notify();
                    }
                }))
                .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                    if this.is_resizing_pane {
                        let new_w: f32 = (event.position.x / px(1.0)).clamp(300.0, 750.0);
                        if (new_w - this.list_pane_width).abs() >= 1.0 {
                            this.list_pane_width = new_w;
                            cx.notify();
                        }
                    }
                }))
                .child(list_pane)
                .child(splitter)
                .child(details_pane)
        };

        // ── Tiroir de logs ───────────────────────────────────────────────────
        let log_drawer = div().child(LogDrawer::render(LogDrawerProps {
            logs: &self.operation_logs,
            status: &self.operation_status,
            is_open: self.log_drawer_open,
            auto_scroll: self.auto_scroll_logs,
            copied_feedback: self.logs_copied_feedback,
            theme: &theme,
            on_toggle: Some(Rc::new(cx.listener(|this, _, _, cx| {
                this.toggle_log_drawer(cx);
            }))),
            on_copy: Some(Rc::new(cx.listener(|this, _, _window, cx| {
                this.copy_logs_to_clipboard(cx);
            }))),
            on_clear: Some(Rc::new(cx.listener(|this, _, _, cx| {
                this.clear_logs(cx);
            }))),
            on_toggle_autoscroll: Some(Rc::new(cx.listener(|this, _, _, cx| {
                this.auto_scroll_logs = !this.auto_scroll_logs;
                cx.notify();
            }))),
        }));

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .child(header)
            .child(div().flex_grow().overflow_hidden().child(content))
            .child(log_drawer)
    }
}

impl WorkspaceView {
    /// Bouton d'onglet simple avec état hover utilisant bg_surface_hover
    fn tab_button<F>(
        label: &'static str,
        is_active: bool,
        theme: &Theme,
        on_click: F,
    ) -> impl IntoElement
    where
        F: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .bg(if is_active {
                theme.bg_surface_active
            } else {
                theme.bg_sidebar
            })
            .when(!is_active, |el| el.hover(|s| s.bg(theme.bg_surface_hover)))
            .text_xs()
            .font_weight(if is_active {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(if is_active {
                theme.text_primary
            } else {
                theme.text_muted
            })
            .cursor_pointer()
            .child(label)
            .on_mouse_down(MouseButton::Left, on_click)
    }

    /// Bouton d'onglet avec badge numérique — utilise StatusPill::badge_count pour le compteur
    fn tab_button_with_badge<F>(
        label: &'static str,
        is_active: bool,
        badge_count: usize,
        theme: &Theme,
        on_click: F,
    ) -> impl IntoElement
    where
        F: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        div()
            .flex()
            .items_center()
            .gap_1p5()
            .px_3()
            .py_1()
            .rounded_md()
            .bg(if is_active {
                theme.bg_surface_active
            } else {
                theme.bg_sidebar
            })
            .when(!is_active, |el| el.hover(|s| s.bg(theme.bg_surface_hover)))
            .text_xs()
            .font_weight(if is_active {
                FontWeight::BOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(if is_active {
                theme.text_primary
            } else {
                theme.text_muted
            })
            .cursor_pointer()
            .child(label)
            .when(badge_count > 0, |el| {
                el.child(StatusPill::badge_count(badge_count, theme))
            })
            .on_mouse_down(MouseButton::Left, on_click)
    }
}
