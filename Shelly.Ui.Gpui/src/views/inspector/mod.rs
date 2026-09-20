pub mod dependencies;
pub mod files_build;
pub mod overview;

use crate::backend::models::{AlpmPackage, UnifiedPackage};
use crate::components::inspector_header::{
    InspectorHeader, InspectorHeaderProps, TabSelectHandler, WindowActionHandler,
};
use crate::components::semantic_value::StringActionHandler;
use crate::state::{InspectorTab, PackageCapabilities};
use crate::theme::Theme;
use crate::views::inspector::dependencies::{DependenciesView, DependenciesViewProps};
use crate::views::inspector::files_build::{FilesBuildView, FilesBuildViewProps};
use crate::views::inspector::overview::{OverviewView, OverviewViewProps};
use gpui::*;

pub struct PackageInspectorProps<'a> {
    pub package: Option<&'a UnifiedPackage>,
    pub alpm_details: Option<&'a AlpmPackage>,
    pub pkgbuild: Option<&'a String>,
    pub is_loading_pkgbuild: bool,
    pub active_tab: InspectorTab,
    pub theme: &'a Theme,
    pub is_busy: bool,
    pub copy_feedback: bool,
    pub on_select_tab: TabSelectHandler,
    pub on_install: Option<WindowActionHandler>,
    pub on_remove: Option<WindowActionHandler>,
    pub on_copy_install_cmd: Option<WindowActionHandler>,
    pub on_copy_pkgbuild: Option<WindowActionHandler>,
    pub on_navigate_package: Option<StringActionHandler>,
}

pub struct PackageInspectorView;

impl PackageInspectorView {
    pub fn render_with_motion(
        props: PackageInspectorProps,
        reduce_motion: bool,
        tab_epoch: u64,
    ) -> AnyElement {
        let theme = props.theme;

        let Some(pkg) = props.package else {
            return div()
                .id("empty_inspector")
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .bg(theme.bg_app)
                .text_color(theme.text_muted)
                .child("Select a package from the list to inspect its details.")
                .into_any_element();
        };

        let capabilities = PackageCapabilities::derive(pkg, props.alpm_details);

        let header = InspectorHeader::render(InspectorHeaderProps {
            package: pkg,
            capabilities: &capabilities,
            active_tab: props.active_tab,
            theme,
            is_busy: props.is_busy,
            copy_feedback: props.copy_feedback,
            on_select_tab: props.on_select_tab.clone(),
            on_install: props.on_install.clone(),
            on_remove: props.on_remove.clone(),
            on_copy_install_cmd: props.on_copy_install_cmd.clone(),
        });

        let body: AnyElement = match props.active_tab {
            InspectorTab::Overview => OverviewView::render(OverviewViewProps {
                package: pkg,
                alpm_details: props.alpm_details,
                theme,
                on_navigate_package: props.on_navigate_package.clone(),
            })
            .into_any_element(),

            InspectorTab::Dependencies => DependenciesView::render(DependenciesViewProps {
                package: pkg,
                alpm_details: props.alpm_details,
                theme,
                on_navigate_package: props.on_navigate_package.clone(),
            })
            .into_any_element(),

            InspectorTab::FilesBuild => FilesBuildView::render(FilesBuildViewProps {
                package: pkg,
                alpm_details: props.alpm_details,
                pkgbuild: props.pkgbuild,
                is_loading_pkgbuild: props.is_loading_pkgbuild,
                theme,
                on_copy_pkgbuild: props.on_copy_pkgbuild.clone(),
            })
            .into_any_element(),
        };

        let animated_body = if reduce_motion {
            div()
                .id("inspector_tab_body")
                .child(body)
                .into_any_element()
        } else {
            div()
                .id("inspector_tab_body")
                .child(body)
                .with_animation(
                    ("tab-fade", tab_epoch as usize),
                    Animation::new(crate::state::MotionDurations::FAST)
                        .with_easing(gpui::ease_out_quint()),
                    |elem, delta| elem.opacity(delta),
                )
                .into_any_element()
        };

        div()
            .id("inspector_scroll")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .p_6()
            .overflow_scroll()
            .child(header)
            .child(animated_body)
            .into_any_element()
    }
}
