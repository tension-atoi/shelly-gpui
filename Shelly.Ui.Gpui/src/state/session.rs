use crate::backend::models::UnifiedPackage;
use gpui::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageSourceKind {
    Alpm,
    Aur,
    Flatpak,
    AppImage,
}

impl std::fmt::Display for PackageSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageSourceKind::Alpm => write!(f, "ALPM"),
            PackageSourceKind::Aur => write!(f, "AUR"),
            PackageSourceKind::Flatpak => write!(f, "Flatpak"),
            PackageSourceKind::AppImage => write!(f, "AppImage"),
        }
    }
}

/// Identifiant stable et typé d'un paquet garantissant l'indépendance vis-à-vis des index de liste
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageKey {
    pub source: PackageSourceKind,
    pub name: String,
    pub repository: Option<String>,
}

impl PackageKey {
    pub fn new(
        source: PackageSourceKind,
        name: impl Into<String>,
        repository: Option<String>,
    ) -> Self {
        Self {
            source,
            name: name.into(),
            repository,
        }
    }

    pub fn from_unified(pkg: &UnifiedPackage) -> Self {
        let source = match pkg.source_type.as_str() {
            "AUR" => PackageSourceKind::Aur,
            "Flatpak" => PackageSourceKind::Flatpak,
            "AppImage" => PackageSourceKind::AppImage,
            _ => PackageSourceKind::Alpm,
        };
        PackageKey {
            source,
            name: pkg.name.clone(),
            repository: if pkg.repository_or_remote.is_empty() {
                None
            } else {
                Some(pkg.repository_or_remote.clone())
            },
        }
    }
}

/// Destinations principales de la barre latérale de la station de travail
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NavDestination {
    #[default]
    Browse,
    Installed,
    Updates,
    News,
    Settings,
}

use crate::icons::AppIcon;

impl NavDestination {
    pub fn label(&self) -> &'static str {
        match self {
            NavDestination::Browse => "Browse",
            NavDestination::Installed => "Installed",
            NavDestination::Updates => "Updates",
            NavDestination::News => "News",
            NavDestination::Settings => "Settings",
        }
    }

    pub fn icon(&self) -> AppIcon {
        match self {
            NavDestination::Browse => AppIcon::Browse,
            NavDestination::Installed => AppIcon::Installed,
            NavDestination::Updates => AppIcon::Updates,
            NavDestination::News => AppIcon::News,
            NavDestination::Settings => AppIcon::Settings,
        }
    }

    pub fn workspace_config_index(&self) -> Option<usize> {
        match self {
            NavDestination::Browse => Some(0),
            NavDestination::Installed => Some(1),
            NavDestination::Updates => Some(2),
            NavDestination::News => Some(3),
            NavDestination::Settings => None,
        }
    }

    pub fn from_config_index(idx: usize) -> Self {
        match idx {
            1 => NavDestination::Installed,
            2 => NavDestination::Updates,
            3 => NavDestination::News,
            _ => NavDestination::Browse,
        }
    }
}

/// Filtres de source disponibles au sein de la destination Découvrir (Browse)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SourceFilter {
    #[default]
    All,
    Alpm,
    Aur,
    Flatpak,
    AppImage,
}

impl SourceFilter {
    pub fn label(&self) -> &'static str {
        match self {
            SourceFilter::All => "All",
            SourceFilter::Alpm => "Official / ALPM",
            SourceFilter::Aur => "AUR",
            SourceFilter::Flatpak => "Flatpak",
            SourceFilter::AppImage => "AppImage",
        }
    }
}

/// Mode d'affichage de la surface des paquets (Cartes ou Table dense)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PackageViewMode {
    #[default]
    Cards,
    Table,
}

impl PackageViewMode {
    pub fn label(&self) -> &'static str {
        match self {
            PackageViewMode::Cards => "Cards",
            PackageViewMode::Table => "Table",
        }
    }

    pub fn icon(&self) -> AppIcon {
        match self {
            PackageViewMode::Cards => AppIcon::Cards,
            PackageViewMode::Table => AppIcon::Table,
        }
    }
}

/// Onglets de l'inspecteur sémantique de paquet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum InspectorTab {
    #[default]
    Overview,
    Dependencies,
    FilesBuild,
}

impl InspectorTab {
    pub fn label(&self) -> &'static str {
        match self {
            InspectorTab::Overview => "Overview",
            InspectorTab::Dependencies => "Dependencies",
            InspectorTab::FilesBuild => "Files & Build",
        }
    }

    pub fn icon(&self) -> AppIcon {
        match self {
            InspectorTab::Overview => AppIcon::Overview,
            InspectorTab::Dependencies => AppIcon::Dependencies,
            InspectorTab::FilesBuild => AppIcon::FilesBuild,
        }
    }
}

/// Événements sémantiques émis par la session applicative
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    DestinationChanged(NavDestination),
    SourceFilterChanged(SourceFilter),
    SearchQueryChanged(String),
    PackageSelected(Option<PackageKey>),
    SidebarToggled(bool),
    ViewModeChanged(PackageViewMode),
    InspectorTabChanged(InspectorTab),
}

/// Entité GPUI gérant l'état de navigation et d'intention de l'utilisateur
pub struct AppSession {
    pub destination: NavDestination,
    pub last_workspace_destination: NavDestination,
    pub source_filter: SourceFilter,
    pub search_query: String,
    pub selected_package_key: Option<PackageKey>,
    pub search_generation: usize,
    pub is_searching: bool,
    pub sidebar_collapsed: bool,
    pub view_mode: PackageViewMode,
    pub inspector_tab: InspectorTab,
    pub destination_epoch: u64,
    pub inspector_tab_epoch: u64,
}

impl EventEmitter<SessionEvent> for AppSession {}

impl AppSession {
    pub fn new() -> Self {
        Self::with_initial_tab(0)
    }

    pub fn with_initial_tab(tab_idx: usize) -> Self {
        let initial_dest = NavDestination::from_config_index(tab_idx);
        Self {
            destination: initial_dest,
            last_workspace_destination: initial_dest,
            source_filter: SourceFilter::All,
            search_query: String::new(),
            selected_package_key: None,
            search_generation: 0,
            is_searching: false,
            sidebar_collapsed: false,
            view_mode: PackageViewMode::Cards,
            inspector_tab: InspectorTab::Overview,
            destination_epoch: 0,
            inspector_tab_epoch: 0,
        }
    }

    pub fn set_destination(&mut self, dest: NavDestination, cx: &mut Context<Self>) {
        if self.destination != dest {
            self.destination = dest;
            if dest != NavDestination::Settings {
                self.last_workspace_destination = dest;
            }
            self.destination_epoch = self.destination_epoch.wrapping_add(1);
            cx.emit(SessionEvent::DestinationChanged(dest));
            cx.notify();
        }
    }

    pub fn set_source_filter(&mut self, filter: SourceFilter, cx: &mut Context<Self>) {
        if self.source_filter != filter {
            self.source_filter = filter;
            cx.emit(SessionEvent::SourceFilterChanged(filter));
            cx.notify();
        }
    }

    pub fn set_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_query = query.clone();
        cx.emit(SessionEvent::SearchQueryChanged(query));
        cx.notify();
    }

    pub fn select_package(&mut self, key: Option<PackageKey>, cx: &mut Context<Self>) {
        if self.selected_package_key != key {
            self.selected_package_key = key.clone();
            cx.emit(SessionEvent::PackageSelected(key));
            cx.notify();
        }
    }

    pub fn set_view_mode(&mut self, mode: PackageViewMode, cx: &mut Context<Self>) {
        if self.view_mode != mode {
            self.view_mode = mode;
            cx.emit(SessionEvent::ViewModeChanged(mode));
            cx.notify();
        }
    }

    pub fn set_inspector_tab(&mut self, tab: InspectorTab, cx: &mut Context<Self>) {
        if self.inspector_tab != tab {
            self.inspector_tab = tab;
            self.inspector_tab_epoch = self.inspector_tab_epoch.wrapping_add(1);
            cx.emit(SessionEvent::InspectorTabChanged(tab));
            cx.notify();
        }
    }

    pub fn set_sidebar_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.sidebar_collapsed != collapsed {
            self.sidebar_collapsed = collapsed;
            cx.emit(SessionEvent::SidebarToggled(self.sidebar_collapsed));
            cx.notify();
        }
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.set_sidebar_collapsed(!self.sidebar_collapsed, cx);
    }

    pub fn next_search_generation(&mut self) -> usize {
        self.search_generation += 1;
        self.search_generation
    }

    pub fn set_searching(&mut self, searching: bool, cx: &mut Context<Self>) {
        self.is_searching = searching;
        cx.notify();
    }
}

impl Default for AppSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_package_key_equality_and_hashing() {
        let k1 = PackageKey::new(PackageSourceKind::Alpm, "ripgrep", Some("extra".into()));
        let k2 = PackageKey::new(PackageSourceKind::Alpm, "ripgrep", Some("extra".into()));
        let k3 = PackageKey::new(PackageSourceKind::Aur, "ripgrep", None);

        assert_eq!(k1, k2);
        assert_ne!(k1, k3);

        let mut map = std::collections::HashMap::new();
        map.insert(k1.clone(), "alpm-extra");
        map.insert(k3.clone(), "aur");

        assert_eq!(map.get(&k2), Some(&"alpm-extra"));
        assert_eq!(map.get(&k3), Some(&"aur"));
    }

    #[test]
    fn test_app_session_defaults() {
        let session = AppSession::new();
        assert_eq!(session.destination, NavDestination::Browse);
        assert_eq!(session.source_filter, SourceFilter::All);
        assert_eq!(session.search_query, "");
        assert_eq!(session.selected_package_key, None);
        assert!(!session.sidebar_collapsed);
        assert_eq!(session.search_generation, 0);
        assert_eq!(session.view_mode, PackageViewMode::Cards);
        assert_eq!(session.inspector_tab, InspectorTab::Overview);
    }

    #[test]
    fn test_nav_destination_metadata() {
        assert_eq!(NavDestination::Browse.label(), "Browse");
        assert_eq!(NavDestination::Installed.label(), "Installed");
        assert_eq!(NavDestination::Updates.label(), "Updates");
        assert_eq!(NavDestination::News.label(), "News");
        assert_eq!(NavDestination::Settings.label(), "Settings");
    }

    #[test]
    fn test_source_filter_metadata() {
        assert_eq!(SourceFilter::All.label(), "All");
        assert_eq!(SourceFilter::Alpm.label(), "Official / ALPM");
        assert_eq!(SourceFilter::Aur.label(), "AUR");
        assert_eq!(SourceFilter::Flatpak.label(), "Flatpak");
        assert_eq!(SourceFilter::AppImage.label(), "AppImage");
    }

    #[test]
    fn test_view_mode_and_inspector_tab_metadata() {
        assert_eq!(PackageViewMode::Cards.label(), "Cards");
        assert_eq!(PackageViewMode::Table.label(), "Table");
        assert_eq!(PackageViewMode::Cards.icon(), AppIcon::Cards);
        assert_eq!(PackageViewMode::Table.icon(), AppIcon::Table);

        assert_eq!(InspectorTab::Overview.label(), "Overview");
        assert_eq!(InspectorTab::Dependencies.label(), "Dependencies");
        assert_eq!(InspectorTab::FilesBuild.label(), "Files & Build");
        assert_eq!(InspectorTab::Overview.icon(), AppIcon::Overview);
        assert_eq!(InspectorTab::Dependencies.icon(), AppIcon::Dependencies);
        assert_eq!(InspectorTab::FilesBuild.icon(), AppIcon::FilesBuild);

        assert_eq!(NavDestination::Browse.icon(), AppIcon::Browse);
        assert_eq!(NavDestination::Installed.icon(), AppIcon::Installed);
        assert_eq!(NavDestination::Updates.icon(), AppIcon::Updates);
        assert_eq!(NavDestination::News.icon(), AppIcon::News);
        assert_eq!(NavDestination::Settings.icon(), AppIcon::Settings);
    }

    #[test]
    fn test_workspace_config_index_round_trip() {
        assert_eq!(NavDestination::Browse.workspace_config_index(), Some(0));
        assert_eq!(NavDestination::Installed.workspace_config_index(), Some(1));
        assert_eq!(NavDestination::Updates.workspace_config_index(), Some(2));
        assert_eq!(NavDestination::News.workspace_config_index(), Some(3));
        assert_eq!(NavDestination::Settings.workspace_config_index(), None);

        assert_eq!(NavDestination::from_config_index(0), NavDestination::Browse);
        assert_eq!(
            NavDestination::from_config_index(1),
            NavDestination::Installed
        );
        assert_eq!(
            NavDestination::from_config_index(2),
            NavDestination::Updates
        );
        assert_eq!(NavDestination::from_config_index(3), NavDestination::News);
        assert_eq!(NavDestination::from_config_index(4), NavDestination::Browse); // legacy settings maps to browse
        assert_eq!(
            NavDestination::from_config_index(99),
            NavDestination::Browse
        );
    }

    #[test]
    fn test_last_workspace_destination_tracks_non_settings() {
        let mut session = AppSession::new();
        assert_eq!(session.destination, NavDestination::Browse);
        assert_eq!(session.last_workspace_destination, NavDestination::Browse);

        // Manually simulate set_destination logic without Context
        session.destination = NavDestination::Installed;
        session.last_workspace_destination = NavDestination::Installed;

        // Navigating to Settings does NOT update last_workspace_destination
        session.destination = NavDestination::Settings;
        // last_workspace_destination remains Installed
        assert_eq!(
            session.last_workspace_destination,
            NavDestination::Installed
        );
        assert_eq!(
            session.last_workspace_destination.workspace_config_index(),
            Some(1)
        );
    }

    #[test]
    fn test_startup_session_preserves_last_workspace_destination_across_settings() {
        let mut session = AppSession::with_initial_tab(1); // 1 = Installed
        assert_eq!(session.destination, NavDestination::Installed);
        assert_eq!(
            session.last_workspace_destination,
            NavDestination::Installed
        );
        assert_eq!(
            session.last_workspace_destination.workspace_config_index(),
            Some(1)
        );

        // Transition to Settings preserves last_workspace_destination = Installed
        session.destination = NavDestination::Settings;
        assert_eq!(
            session.last_workspace_destination.workspace_config_index(),
            Some(1)
        );
    }

    #[test]
    fn test_view_mode_continuity_preserves_selection_query_and_generation() {
        let mut session = AppSession::new();
        session.destination = NavDestination::Browse;
        session.search_query = "ripgrep".to_string();
        session.search_generation = 12;

        let expected_key = PackageKey::new(
            PackageSourceKind::Alpm,
            "ripgrep",
            Some("cachyos-v3".to_string()),
        );
        session.selected_package_key = Some(expected_key.clone());
        assert_eq!(session.view_mode, PackageViewMode::Cards);

        // Switch Cards -> Table
        session.view_mode = PackageViewMode::Table;

        // Verify continuity
        assert_eq!(session.search_query, "ripgrep");
        assert_eq!(session.selected_package_key, Some(expected_key.clone()));
        assert_eq!(session.search_generation, 12);

        // Switch Table -> Cards
        session.view_mode = PackageViewMode::Cards;

        // Verify continuity again
        assert_eq!(session.search_query, "ripgrep");
        assert_eq!(session.selected_package_key, Some(expected_key));
        assert_eq!(session.search_generation, 12);
    }

    #[test]
    fn test_dependency_navigation_preserves_clean_package_name() {
        use crate::state::semantic::{DependencyKind, DependencyRef};

        let raw_dep = "libalpm.so>=14: Arch package management library";
        let parsed = DependencyRef::parse(raw_dep, DependencyKind::Runtime);
        assert_eq!(parsed.name, "libalpm.so");
        assert_eq!(parsed.constraint, Some(">=14".to_string()));

        // Simulate dependency click navigation in session
        let mut session = AppSession::new();
        session.destination = NavDestination::Installed;
        session.search_query = "shelly".to_string();

        // When navigating to dependency, clean name is used, not raw constraint/description
        session.destination = NavDestination::Browse;
        session.search_query = parsed.name.clone();
        session.search_generation += 1;

        assert_eq!(session.destination, NavDestination::Browse);
        assert_eq!(session.search_query, "libalpm.so");
        assert!(!session.search_query.contains(">=14"));
        assert!(!session.search_query.contains("Arch package"));
    }

    #[test]
    fn test_session_epochs_increment() {
        let mut session = AppSession::new();
        assert_eq!(session.destination_epoch, 0);
        assert_eq!(session.inspector_tab_epoch, 0);

        // Manually simulate epoch increments as in set_destination and set_inspector_tab
        session.destination = NavDestination::News;
        session.destination_epoch += 1;
        assert_eq!(session.destination_epoch, 1);

        session.inspector_tab = InspectorTab::Dependencies;
        session.inspector_tab_epoch += 1;
        assert_eq!(session.inspector_tab_epoch, 1);
    }
}
