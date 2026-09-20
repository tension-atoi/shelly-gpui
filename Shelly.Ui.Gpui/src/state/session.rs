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

    pub fn icon(&self) -> &'static str {
        match self {
            NavDestination::Browse => "🔍",
            NavDestination::Installed => "📦",
            NavDestination::Updates => "🔄",
            NavDestination::News => "📰",
            NavDestination::Settings => "⚙️",
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

    pub fn icon(&self) -> &'static str {
        match self {
            PackageViewMode::Cards => "▦",
            PackageViewMode::Table => "☰",
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

    pub fn icon(&self) -> &'static str {
        match self {
            InspectorTab::Overview => "📋",
            InspectorTab::Dependencies => "🔗",
            InspectorTab::FilesBuild => "🛠️",
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
    pub source_filter: SourceFilter,
    pub search_query: String,
    pub selected_package_key: Option<PackageKey>,
    pub search_generation: usize,
    pub is_searching: bool,
    pub sidebar_collapsed: bool,
    pub view_mode: PackageViewMode,
    pub inspector_tab: InspectorTab,
}

impl EventEmitter<SessionEvent> for AppSession {}

impl AppSession {
    pub fn new() -> Self {
        Self {
            destination: NavDestination::Browse,
            source_filter: SourceFilter::All,
            search_query: String::new(),
            selected_package_key: None,
            search_generation: 0,
            is_searching: false,
            sidebar_collapsed: false,
            view_mode: PackageViewMode::Cards,
            inspector_tab: InspectorTab::Overview,
        }
    }

    pub fn set_destination(&mut self, dest: NavDestination, cx: &mut Context<Self>) {
        if self.destination != dest {
            self.destination = dest;
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
            cx.emit(SessionEvent::InspectorTabChanged(tab));
            cx.notify();
        }
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.emit(SessionEvent::SidebarToggled(self.sidebar_collapsed));
        cx.notify();
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
        assert_eq!(PackageViewMode::Cards.icon(), "▦");
        assert_eq!(PackageViewMode::Table.icon(), "☰");

        assert_eq!(InspectorTab::Overview.label(), "Overview");
        assert_eq!(InspectorTab::Dependencies.label(), "Dependencies");
        assert_eq!(InspectorTab::FilesBuild.label(), "Files & Build");
        assert_eq!(InspectorTab::Overview.icon(), "📋");
        assert_eq!(InspectorTab::Dependencies.icon(), "🔗");
        assert_eq!(InspectorTab::FilesBuild.icon(), "🛠️");
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
}
