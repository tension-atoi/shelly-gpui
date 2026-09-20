use crate::backend::models::{
    AlpmPackage, AurPackage, FlatpakHit, PackageUpdateItem, UnifiedPackage,
};
use gpui::{Context, EventEmitter};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFilter {
    All,
    Official,
    Aur,
    Flatpak,
}

#[derive(Debug, Clone)]
pub enum CatalogEvent {
    SearchStarted { query: String },
    SearchResultsUpdated { total: usize, official: usize, aur: usize, flatpak: usize },
    PackageSelected(Option<UnifiedPackage>),
    SourceFilterChanged(SourceFilter),
    UpdatesLoaded(usize),
    CacheInvalidated,
}

pub struct CatalogModel {
    pub search_query: String,
    pub source_filter: SourceFilter,
    pub is_searching: bool,
    pub official_results: Vec<UnifiedPackage>,
    pub aur_results: Vec<UnifiedPackage>,
    pub flatpak_results: Vec<UnifiedPackage>,
    pub updates: Vec<PackageUpdateItem>,
    pub selected_package: Option<UnifiedPackage>,
    pub detail_cache: HashMap<String, AlpmPackage>,
}

impl EventEmitter<CatalogEvent> for CatalogModel {}

impl Default for CatalogModel {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogModel {
    pub fn new() -> Self {
        Self {
            search_query: String::new(),
            source_filter: SourceFilter::All,
            is_searching: false,
            official_results: Vec::new(),
            aur_results: Vec::new(),
            flatpak_results: Vec::new(),
            updates: Vec::new(),
            selected_package: None,
            detail_cache: HashMap::new(),
        }
    }

    pub fn set_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        if self.search_query == query {
            return;
        }
        self.search_query = query.clone();
        self.is_searching = !query.trim().is_empty();
        cx.emit(CatalogEvent::SearchStarted { query });
        cx.notify();
    }

    pub fn set_source_filter(&mut self, filter: SourceFilter, cx: &mut Context<Self>) {
        if self.source_filter == filter {
            return;
        }
        self.source_filter = filter;
        cx.emit(CatalogEvent::SourceFilterChanged(filter));
        cx.notify();
    }

    pub fn set_search_results(
        &mut self,
        official: Vec<AlpmPackage>,
        aur: Vec<AurPackage>,
        flatpak: Vec<FlatpakHit>,
        cx: &mut Context<Self>,
    ) {
        self.is_searching = false;
        self.official_results = official
            .into_iter()
            .map(|p| {
                let is_inst = p.install_date.is_some() || p.install_reason.is_some();
                UnifiedPackage::from_alpm(p, is_inst)
            })
            .collect();
        self.aur_results = aur
            .into_iter()
            .map(|p| UnifiedPackage::from_aur(p, false))
            .collect();
        self.flatpak_results = flatpak
            .into_iter()
            .map(|h| {
                let is_inst = false;
                UnifiedPackage::from_flatpak(h, is_inst)
            })
            .collect();

        let off_count = self.official_results.len();
        let aur_count = self.aur_results.len();
        let fp_count = self.flatpak_results.len();
        let total = off_count + aur_count + fp_count;

        let items = self.filtered_items();
        self.selected_package = items.first().cloned();

        cx.emit(CatalogEvent::SearchResultsUpdated {
            total,
            official: off_count,
            aur: aur_count,
            flatpak: fp_count,
        });
        cx.emit(CatalogEvent::PackageSelected(self.selected_package.clone()));
        cx.notify();
    }


    pub fn set_updates(&mut self, updates: Vec<PackageUpdateItem>, cx: &mut Context<Self>) {
        let count = updates.len();
        self.updates = updates;
        cx.emit(CatalogEvent::UpdatesLoaded(count));
        cx.notify();
    }

    pub fn select_package(&mut self, package: Option<UnifiedPackage>, cx: &mut Context<Self>) {
        self.selected_package = package.clone();
        cx.emit(CatalogEvent::PackageSelected(package));
        cx.notify();
    }

    pub fn invalidate_cache(&mut self, cx: &mut Context<Self>) {
        self.detail_cache.clear();
        cx.emit(CatalogEvent::CacheInvalidated);
        cx.notify();
    }

    pub fn filtered_items(&self) -> Vec<UnifiedPackage> {
        match self.source_filter {
            SourceFilter::All => {
                let mut all = Vec::with_capacity(
                    self.official_results.len() + self.aur_results.len() + self.flatpak_results.len(),
                );
                all.extend(self.official_results.clone());
                all.extend(self.aur_results.clone());
                all.extend(self.flatpak_results.clone());
                all
            }
            SourceFilter::Official => self.official_results.clone(),
            SourceFilter::Aur => self.aur_results.clone(),
            SourceFilter::Flatpak => self.flatpak_results.clone(),
        }
    }

    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let off = self.official_results.len();
        let aur = self.aur_results.len();
        let fp = self.flatpak_results.len();
        (off + aur + fp, off, aur, fp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_model_default() {
        let model = CatalogModel::new();
        assert_eq!(model.search_query, "");
        assert_eq!(model.source_filter, SourceFilter::All);
        assert!(!model.is_searching);
        assert_eq!(model.counts(), (0, 0, 0, 0));
        assert!(model.filtered_items().is_empty());
    }
}
