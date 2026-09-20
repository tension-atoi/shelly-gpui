use gpui::{Context, EventEmitter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavRoute {
    Search,
    Updates,
    Installed,
    Settings,
}

impl NavRoute {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Search => "Recherche",
            Self::Updates => "Mises à jour",
            Self::Installed => "Installés",
            Self::Settings => "Paramètres",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Search => "🔍",
            Self::Updates => "🔄",
            Self::Installed => "📦",
            Self::Settings => "⚙️",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageViewMode {
    Cards,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorTab {
    Overview,
    Dependencies,
    Files,
}

impl InspectorTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Overview => "Aperçu",
            Self::Dependencies => "Dépendances",
            Self::Files => "Fichiers",
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayoutEvent {
    RouteChanged(NavRoute),
    SidebarToggled(bool),
    ViewModeChanged(PackageViewMode),
    InspectorTabChanged(InspectorTab),
    PaneResized(f32),
    DiagnosticsHudToggled(bool),
    MetricsUpdated { fps: f32, frame_time_ms: f32 },
}

pub struct LayoutModel {
    pub active_route: NavRoute,
    pub is_sidebar_collapsed: bool,
    pub package_view_mode: PackageViewMode,
    pub active_inspector_tab: InspectorTab,
    pub list_pane_width: f32,
    pub show_diagnostics_hud: bool,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub active_rows: usize,
}

impl EventEmitter<LayoutEvent> for LayoutModel {}

impl Default for LayoutModel {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutModel {
    pub fn new() -> Self {
        Self {
            active_route: NavRoute::Search,
            is_sidebar_collapsed: true, // 56px icon rail by default
            package_view_mode: PackageViewMode::Cards,
            active_inspector_tab: InspectorTab::Overview,
            list_pane_width: 380.0,
            show_diagnostics_hud: false,
            fps: 60.0,
            frame_time_ms: 16.6,
            active_rows: 0,
        }
    }

    pub fn set_route(&mut self, route: NavRoute, cx: &mut Context<Self>) {
        if self.active_route == route {
            return;
        }
        self.active_route = route;
        cx.emit(LayoutEvent::RouteChanged(route));
        cx.notify();
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.is_sidebar_collapsed = !self.is_sidebar_collapsed;
        cx.emit(LayoutEvent::SidebarToggled(self.is_sidebar_collapsed));
        cx.notify();
    }

    pub fn set_view_mode(&mut self, mode: PackageViewMode, cx: &mut Context<Self>) {
        if self.package_view_mode == mode {
            return;
        }
        self.package_view_mode = mode;
        cx.emit(LayoutEvent::ViewModeChanged(mode));
        cx.notify();
    }

    pub fn set_inspector_tab(&mut self, tab: InspectorTab, cx: &mut Context<Self>) {
        if self.active_inspector_tab == tab {
            return;
        }
        self.active_inspector_tab = tab;
        cx.emit(LayoutEvent::InspectorTabChanged(tab));
        cx.notify();
    }

    pub fn set_list_pane_width(&mut self, width: f32, cx: &mut Context<Self>) {
        let clamped = width.clamp(240.0, 800.0);
        if (self.list_pane_width - clamped).abs() >= 1.0 {
            self.list_pane_width = clamped;
            cx.emit(LayoutEvent::PaneResized(clamped));
            cx.notify();
        }
    }

    pub fn toggle_diagnostics_hud(&mut self, cx: &mut Context<Self>) {
        self.show_diagnostics_hud = !self.show_diagnostics_hud;
        cx.emit(LayoutEvent::DiagnosticsHudToggled(self.show_diagnostics_hud));
        cx.notify();
    }

    pub fn update_metrics(
        &mut self,
        fps: f32,
        frame_time_ms: f32,
        active_rows: usize,
        cx: &mut Context<Self>,
    ) {
        self.fps = fps;
        self.frame_time_ms = frame_time_ms;
        self.active_rows = active_rows;
        cx.emit(LayoutEvent::MetricsUpdated { fps, frame_time_ms });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_model_default() {
        let model = LayoutModel::new();
        assert_eq!(model.active_route, NavRoute::Search);
        assert!(model.is_sidebar_collapsed);
        assert_eq!(model.package_view_mode, PackageViewMode::Cards);
        assert_eq!(model.active_inspector_tab, InspectorTab::Overview);
        assert_eq!(model.list_pane_width, 380.0);
        assert!(!model.show_diagnostics_hud);
        assert_eq!(model.fps, 60.0);
        assert_eq!(model.frame_time_ms, 16.6);
        assert_eq!(model.active_rows, 0);
    }

    #[test]
    fn test_nav_route_metadata() {
        assert_eq!(NavRoute::Search.label(), "Recherche");
        assert_eq!(NavRoute::Updates.label(), "Mises à jour");
        assert_eq!(NavRoute::Installed.label(), "Installés");
        assert_eq!(NavRoute::Settings.label(), "Paramètres");
    }

    #[test]
    fn test_inspector_tab_metadata() {
        assert_eq!(InspectorTab::Overview.label(), "Aperçu");
        assert_eq!(InspectorTab::Dependencies.label(), "Dépendances");
        assert_eq!(InspectorTab::Files.label(), "Fichiers");
    }
}
