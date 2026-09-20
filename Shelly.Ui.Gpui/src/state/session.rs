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

/// Événements sémantiques émis par la session applicative
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    DestinationChanged(NavDestination),
    SourceFilterChanged(SourceFilter),
    SearchQueryChanged(String),
    PackageSelected(Option<PackageKey>),
    SidebarToggled(bool),
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
}
