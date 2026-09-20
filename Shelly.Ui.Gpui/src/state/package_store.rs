use crate::backend::client::ShellyClient;
use crate::backend::models::{AlpmPackage, UnifiedPackage};
use crate::state::session::{PackageKey, SourceFilter};
use gpui::*;
use std::collections::HashMap;

/// Clé d'indexation pour le cache de recherche en session
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchKey {
    pub query: String,
    pub source_filter: SourceFilter,
}

impl SearchKey {
    pub fn new(query: impl Into<String>, source_filter: SourceFilter) -> Self {
        Self {
            query: query.into().trim().to_lowercase(),
            source_filter,
        }
    }
}

/// Événements sémantiques émis par le magasin de paquets
#[derive(Debug, Clone, PartialEq)]
pub enum PackageStoreEvent {
    ResultsChanged,
    PackageInvalidated(PackageKey),
    UpdatesChanged(usize),
    InstalledChanged(usize),
}

/// Magasin d'état applicatif gérant les collections de paquets, caches et invalidations
pub struct PackageStore {
    pub client: ShellyClient,
    pub active_results: Vec<UnifiedPackage>,
    pub installed_packages: Vec<UnifiedPackage>,
    pub updates_packages: Vec<UnifiedPackage>,
    pub updates_count: usize,
    pub detail_cache: HashMap<PackageKey, AlpmPackage>,
    pub search_cache: HashMap<SearchKey, Vec<UnifiedPackage>>,
    pub pkgbuild_cache: HashMap<String, String>,
    pub in_flight_generation: usize,
}

impl EventEmitter<PackageStoreEvent> for PackageStore {}

impl PackageStore {
    pub fn new(client: ShellyClient) -> Self {
        Self {
            client,
            active_results: Vec::new(),
            installed_packages: Vec::new(),
            updates_packages: Vec::new(),
            updates_count: 0,
            detail_cache: HashMap::new(),
            search_cache: HashMap::new(),
            pkgbuild_cache: HashMap::new(),
            in_flight_generation: 0,
        }
    }

    /// Tente de récupérer les résultats de recherche depuis le cache de session
    pub fn get_cached_search(
        &self,
        query: &str,
        filter: SourceFilter,
    ) -> Option<&Vec<UnifiedPackage>> {
        let key = SearchKey::new(query, filter);
        self.search_cache.get(&key)
    }

    /// Enregistre des résultats dans le cache de recherche de session
    pub fn cache_search(
        &mut self,
        query: &str,
        filter: SourceFilter,
        results: Vec<UnifiedPackage>,
    ) {
        let key = SearchKey::new(query, filter);
        self.search_cache.insert(key, results);
    }

    /// Met à jour les résultats actifs affichés
    pub fn set_active_results(
        &mut self,
        results: Vec<UnifiedPackage>,
        generation: usize,
        cx: &mut Context<Self>,
    ) {
        if generation >= self.in_flight_generation {
            self.in_flight_generation = generation;
            self.active_results = results;
            cx.emit(PackageStoreEvent::ResultsChanged);
            cx.notify();
        }
    }

    /// Définit la liste des paquets installés
    pub fn set_installed_packages(&mut self, pkgs: Vec<UnifiedPackage>, cx: &mut Context<Self>) {
        let count = pkgs.len();
        self.installed_packages = pkgs;
        cx.emit(PackageStoreEvent::InstalledChanged(count));
        cx.notify();
    }

    /// Définit la liste des mises à jour disponibles
    pub fn set_updates_packages(&mut self, pkgs: Vec<UnifiedPackage>, cx: &mut Context<Self>) {
        self.updates_count = pkgs.len();
        self.updates_packages = pkgs;
        cx.emit(PackageStoreEvent::UpdatesChanged(self.updates_count));
        cx.notify();
    }

    /// Récupère la fiche détaillée d'un paquet depuis le cache
    pub fn get_cached_details(&self, key: &PackageKey) -> Option<&AlpmPackage> {
        self.detail_cache.get(key)
    }

    /// Enregistre une fiche détaillée dans le cache
    pub fn cache_details(&mut self, key: PackageKey, details: AlpmPackage) {
        self.detail_cache.insert(key, details);
    }

    /// Récupère le PKGBUILD mis en cache pour un paquet AUR
    pub fn get_cached_pkgbuild(&self, name: &str) -> Option<&String> {
        self.pkgbuild_cache.get(name)
    }

    /// Met en cache le contenu d'un PKGBUILD
    pub fn cache_pkgbuild(&mut self, name: String, content: String) {
        self.pkgbuild_cache.insert(name, content);
    }

    /// Invalide un paquet spécifique suite à une mutation (install/remove/update)
    pub fn invalidate_package(&mut self, key: &PackageKey, cx: &mut Context<Self>) {
        self.detail_cache.remove(key);
        self.pkgbuild_cache.remove(&key.name);
        // Supprime les entrées de cache de recherche contenant ce paquet pour éviter les incohérences d'état
        self.search_cache
            .retain(|_, list| !list.iter().any(|p| p.name == key.name));
        cx.emit(PackageStoreEvent::PackageInvalidated(key.clone()));
        cx.notify();
    }

    /// Invalide le cache des paquets installés
    pub fn invalidate_installed(&mut self, cx: &mut Context<Self>) {
        self.installed_packages.clear();
        self.search_cache.clear();
        cx.emit(PackageStoreEvent::InstalledChanged(0));
        cx.notify();
    }

    /// Invalide le cache des mises à jour
    pub fn invalidate_updates(&mut self, cx: &mut Context<Self>) {
        self.updates_packages.clear();
        self.updates_count = 0;
        cx.emit(PackageStoreEvent::UpdatesChanged(0));
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::session::PackageSourceKind;
    use core::prelude::v1::test;

    #[test]
    fn test_search_key_normalization() {
        let k1 = SearchKey::new("  RipGrep  ", SourceFilter::All);
        let k2 = SearchKey::new("ripgrep", SourceFilter::All);
        let k3 = SearchKey::new("ripgrep", SourceFilter::Aur);

        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn test_package_store_cache_and_invalidation() {
        let client = ShellyClient::new(None);
        let mut store = PackageStore::new(client);

        let key = PackageKey::new(PackageSourceKind::Alpm, "neovim", Some("extra".into()));
        let mut details = AlpmPackage::default();
        details.name = "neovim".into();
        details.version = "0.10.0".into();

        store.cache_details(key.clone(), details.clone());
        assert!(store.get_cached_details(&key).is_some());

        // Invalidation manuelle du cache
        store.detail_cache.remove(&key);
        assert!(store.get_cached_details(&key).is_none());
    }

    #[test]
    fn test_search_cache_storage() {
        let client = ShellyClient::new(None);
        let mut store = PackageStore::new(client);

        let query = "firefox";
        let filter = SourceFilter::Alpm;

        assert!(store.get_cached_search(query, filter).is_none());

        let pkg = UnifiedPackage {
            name: "firefox".into(),
            version: "128.0".into(),
            description: "Web browser".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: crate::backend::models::UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        store.cache_search(query, filter, vec![pkg]);
        let cached = store.get_cached_search(query, filter);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
        assert_eq!(cached.unwrap()[0].name, "firefox");
    }

    #[test]
    fn test_appimage_filtering_and_key_identity() {
        use crate::backend::models::AppImageItem;

        let item = AppImageItem {
            name: "Obsidian".into(),
            desktop_name: Some("obsidian.desktop".into()),
            version: Some("1.5.8".into()),
            icon_name: Some("obsidian".into()),
            description: Some("Knowledge base and note-taking app".into()),
            size_on_disk: Some(120_000_000),
            update_url: None,
            repo_owner: None,
            repo_name: None,
            path: Some("/home/user/Applications/Obsidian.AppImage".into()),
        };

        let pkg = UnifiedPackage::from_appimage(item.clone());
        assert_eq!(pkg.name, "Obsidian");
        assert_eq!(pkg.version, "1.5.8");
        assert_eq!(pkg.source_type, "AppImage");
        assert_eq!(pkg.repository_or_remote, "appimage");
        assert_eq!(pkg.key().source, PackageSourceKind::AppImage);
        assert_eq!(pkg.key().name, "Obsidian");

        // Test metadata filtering logic for non-empty queries
        let q1 = "obsidian";
        let q2 = "knowledge";
        let q3 = "vlc";

        let matches_q1 = item.name.to_lowercase().contains(q1)
            || item
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(q1))
                .unwrap_or(false);
        let matches_q2 = item.name.to_lowercase().contains(q2)
            || item
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(q2))
                .unwrap_or(false);
        let matches_q3 = item.name.to_lowercase().contains(q3)
            || item
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(q3))
                .unwrap_or(false);

        assert!(matches_q1);
        assert!(matches_q2);
        assert!(!matches_q3);

        // Test storing and retrieving in search cache
        let client = ShellyClient::new(None);
        let mut store = PackageStore::new(client);
        store.cache_search("obsidian", SourceFilter::AppImage, vec![pkg.clone()]);
        let cached = store.get_cached_search("obsidian", SourceFilter::AppImage);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()[0].name, "Obsidian");
        assert_eq!(cached.unwrap()[0].source_type, "AppImage");
    }

    #[test]
    fn test_pkgbuild_cache_storage_and_invalidation() {
        let client = ShellyClient::new(None);
        let mut store = PackageStore::new(client);

        let pkg_name = "visual-studio-code-bin";
        let pkgbuild = "pkgname=visual-studio-code-bin\npkgver=1.138.0";

        assert!(store.get_cached_pkgbuild(pkg_name).is_none());

        store.cache_pkgbuild(pkg_name.into(), pkgbuild.into());
        assert_eq!(
            store.get_cached_pkgbuild(pkg_name),
            Some(&pkgbuild.to_string())
        );

        // Invalidate via key
        let key = PackageKey::new(PackageSourceKind::Aur, pkg_name, None);
        store.pkgbuild_cache.remove(&key.name);
        assert!(store.get_cached_pkgbuild(pkg_name).is_none());
    }
}
