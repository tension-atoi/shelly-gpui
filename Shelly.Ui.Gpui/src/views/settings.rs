use crate::config::{ConfigManager, GpuiUiConfig, ShellySettings};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct SettingsViewProps<'a> {
    pub shelly_settings: &'a ShellySettings,
    pub gpui_config: &'a GpuiUiConfig,
    pub is_dirty: bool,
    pub theme: &'a Theme,
    pub on_toggle_aur: WindowActionHandler,
    pub on_toggle_flatpak: WindowActionHandler,
    pub on_toggle_appimage: WindowActionHandler,
    pub on_toggle_cascade_delete: WindowActionHandler,
    pub on_toggle_remove_configs: WindowActionHandler,
    pub on_toggle_dark_theme: WindowActionHandler,
    pub on_toggle_compact_view: WindowActionHandler,
    pub on_toggle_log_drawer_auto_open: WindowActionHandler,
    pub on_toggle_reduce_motion: WindowActionHandler,
    pub on_save: WindowActionHandler,
    pub on_reset: Option<WindowActionHandler>,
}

#[derive(Debug, Clone)]
pub struct SettingsView {
    pub draft_shelly: ShellySettings,
    pub draft_gpui: GpuiUiConfig,
    pub is_dirty: bool,
}

impl SettingsView {
    pub fn new(shelly_settings: ShellySettings, gpui_config: GpuiUiConfig) -> Self {
        Self {
            draft_shelly: shelly_settings,
            draft_gpui: gpui_config,
            is_dirty: false,
        }
    }

    pub fn toggle_aur(&mut self) {
        self.draft_shelly.aur_enabled = !self.draft_shelly.aur_enabled;
        self.is_dirty = true;
    }

    pub fn toggle_flatpak(&mut self) {
        self.draft_shelly.flat_pack_enabled = !self.draft_shelly.flat_pack_enabled;
        self.is_dirty = true;
    }

    pub fn toggle_appimage(&mut self) {
        self.draft_shelly.app_image_enabled = !self.draft_shelly.app_image_enabled;
        self.is_dirty = true;
    }

    pub fn toggle_cascade_delete(&mut self) {
        self.draft_shelly.package_management_cascade_delete =
            !self.draft_shelly.package_management_cascade_delete;
        self.is_dirty = true;
    }

    pub fn toggle_remove_configs(&mut self) {
        self.draft_shelly.package_management_remove_configs =
            !self.draft_shelly.package_management_remove_configs;
        self.is_dirty = true;
    }

    pub fn toggle_dark_theme(&mut self) {
        self.draft_gpui.dark_theme = !self.draft_gpui.dark_theme;
        self.is_dirty = true;
    }

    pub fn toggle_compact_view(&mut self) {
        self.draft_gpui.compact_view = !self.draft_gpui.compact_view;
        self.is_dirty = true;
    }

    pub fn toggle_log_drawer_auto_open(&mut self) {
        self.draft_gpui.log_drawer_open = !self.draft_gpui.log_drawer_open;
        self.is_dirty = true;
    }

    pub fn toggle_reduce_motion(&mut self) {
        self.draft_gpui.reduce_motion = !self.draft_gpui.reduce_motion;
        self.is_dirty = true;
    }

    pub fn reset_to(&mut self, shelly: ShellySettings, gpui: GpuiUiConfig) {
        self.draft_shelly = shelly;
        self.draft_gpui = gpui;
        self.is_dirty = false;
    }

    /// Sauvegarde pure paramétrable par injection de closures (permettant les tests unitaires sans disque)
    pub fn save_with<F1, F2>(&mut self, save_shelly: F1, save_gpui: F2) -> anyhow::Result<()>
    where
        F1: FnOnce(&ShellySettings) -> anyhow::Result<()>,
        F2: FnOnce(&GpuiUiConfig) -> anyhow::Result<()>,
    {
        save_shelly(&self.draft_shelly)?;
        save_gpui(&self.draft_gpui)?;
        self.is_dirty = false;
        Ok(())
    }

    /// Sauvegarde effective vers le disque via ConfigManager
    pub fn save(&mut self) -> anyhow::Result<()> {
        self.save_with(
            ConfigManager::save_shelly_settings,
            ConfigManager::save_gpui_config,
        )
    }

    pub fn render(props: SettingsViewProps) -> impl IntoElement {
        let theme = props.theme;
        let s = props.shelly_settings;
        let g = props.gpui_config;
        let is_dirty = props.is_dirty;

        let mut root = div()
            .id("settings_scroll")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .p_6()
            .overflow_scroll();

        // En-tête des paramètres
        root = root.child(
            div()
                .flex()
                .flex_col()
                .mb_6()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_2xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child("Shelly Settings"),
                        )
                        .child(if is_dirty {
                            div()
                                .px_2p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme.warning)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.bg_app)
                                .child("Unsaved Changes")
                        } else {
                            div()
                                .px_2p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme.bg_surface)
                                .border_1()
                                .border_color(theme.border)
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text_muted)
                                .child("Up to Date")
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Configuration of package sources, desktop preferences, and motion policy."),
                ),
        );

        // Section 1: PACKAGE SOURCES
        let sources_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("PACKAGE SOURCES"),
            )
            .child(Self::toggle_row(
                "Arch User Repository (AUR)",
                "Enable search and PKGBUILD inspection for community AUR packages",
                s.aur_enabled,
                props.on_toggle_aur,
                theme,
            ))
            .child(Self::toggle_row(
                "Flatpak Packages",
                "Enable search and inspection for sandboxed applications via Flathub",
                s.flat_pack_enabled,
                props.on_toggle_flatpak,
                theme,
            ))
            .child(Self::toggle_row(
                "AppImage Packages",
                "Enable discovery and inspection for self-contained AppImage binaries",
                s.app_image_enabled,
                props.on_toggle_appimage,
                theme,
            ));

        root = root.child(sources_section);

        // Section 2: PACKAGE MANAGEMENT
        let mgmt_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("PACKAGE MANAGEMENT"),
            )
            .child(Self::toggle_row(
                "Cascade Dependency Removal (--cascade)",
                "Recursively remove orphaned dependencies when removing packages",
                s.package_management_cascade_delete,
                props.on_toggle_cascade_delete,
                theme,
            ))
            .child(Self::toggle_row(
                "Remove Configuration Files (--remove-config)",
                "Purge package configuration and backup files upon package removal",
                s.package_management_remove_configs,
                props.on_toggle_remove_configs,
                theme,
            ));

        root = root.child(mgmt_section);

        // Section 3: APPEARANCE & DENSITY
        let appearance_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("APPEARANCE & DENSITY"),
            )
            .child(Self::toggle_row(
                "Dark Theme",
                "Use high-contrast dark theme interface (uncheck for light theme)",
                g.dark_theme,
                props.on_toggle_dark_theme,
                theme,
            ))
            .child(Self::toggle_row(
                "Compact View Density",
                "Reduce card and table row heights and collapse the navigation sidebar",
                g.compact_view,
                props.on_toggle_compact_view,
                theme,
            ));

        root = root.child(appearance_section);

        // Section 4: MOTION & FEEDBACK
        let motion_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("MOTION & FEEDBACK"),
            )
            .child(Self::toggle_row(
                "Reduce Motion",
                "Disable animated transitions and snap sidebar, console, and toasts immediately",
                g.reduce_motion,
                props.on_toggle_reduce_motion,
                theme,
            ));

        root = root.child(motion_section);

        // Section 5: LOGS & OPERATIONS
        let logs_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("LOGS & OPERATIONS"),
            )
            .child(Self::toggle_row(
                "Auto-Open Operation Console",
                "Automatically reveal the log drawer whenever a package mutation starts",
                g.log_drawer_open,
                props.on_toggle_log_drawer_auto_open,
                theme,
            ));

        root = root.child(logs_section);

        // Bouton de sauvegarde explicite & bouton de réinitialisation
        let on_save = props.on_save;
        let on_reset = props.on_reset;
        let mut actions_bar = div().flex().items_center().justify_end().gap_3().mt_6();

        if is_dirty {
            if let Some(reset_handler) = on_reset {
                actions_bar = actions_bar.child(
                    div()
                        .id("reset_settings_btn")
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(theme.bg_surface)
                        .border_1()
                        .border_color(theme.border)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_secondary)
                        .cursor_pointer()
                        .hover(move |s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
                        .child("Reset Changes")
                        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                            reset_handler(window, cx);
                        }),
                );
            }

            actions_bar = actions_bar.child(
                div()
                    .id("save_settings_btn")
                    .px_6()
                    .py_2()
                    .rounded_md()
                    .bg(theme.accent)
                    .border_1()
                    .border_color(theme.accent)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.bg_app)
                    .cursor_pointer()
                    .hover(move |s| s.bg(theme.accent_hover).text_color(theme.bg_app))
                    .child("Save Settings *")
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        on_save(window, cx);
                    }),
            );
        } else {
            actions_bar = actions_bar.child(
                div()
                    .id("save_settings_btn")
                    .px_6()
                    .py_2()
                    .rounded_md()
                    .bg(theme.bg_surface_active)
                    .border_1()
                    .border_color(theme.border)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child("Settings Saved"),
            );
        }

        root = root.child(actions_bar);
        root
    }

    fn toggle_row(
        title: &'static str,
        description: &'static str,
        active: bool,
        on_toggle: WindowActionHandler,
        theme: &Theme,
    ) -> impl IntoElement {
        let hover_bg = theme.bg_surface_hover;
        div()
            .flex()
            .items_center()
            .justify_between()
            .py_3()
            .px_3()
            .rounded_sm()
            .border_b_1()
            .border_color(theme.border)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_secondary)
                            .child(description),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_full()
                    .bg(if active { theme.success } else { theme.border })
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .text_color(if active {
                        theme.bg_app
                    } else {
                        theme.text_muted
                    })
                    .child(if active { "Enabled" } else { "Disabled" })
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        on_toggle(window, cx);
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_settings_draft_dirty_flag() {
        let mut settings = SettingsView::new(ShellySettings::default(), GpuiUiConfig::default());
        assert!(!settings.is_dirty);

        settings.toggle_aur();
        assert!(settings.is_dirty);
        assert!(!settings.draft_shelly.aur_enabled);

        settings.toggle_flatpak();
        assert!(settings.is_dirty);
        assert!(!settings.draft_shelly.flat_pack_enabled);

        settings.toggle_appimage();
        assert!(!settings.draft_shelly.app_image_enabled);

        settings.toggle_cascade_delete();
        assert!(!settings.draft_shelly.package_management_cascade_delete);

        settings.toggle_remove_configs();
        assert!(!settings.draft_shelly.package_management_remove_configs);

        settings.toggle_dark_theme();
        assert!(!settings.draft_gpui.dark_theme);

        settings.toggle_compact_view();
        assert!(settings.draft_gpui.compact_view);

        settings.toggle_log_drawer_auto_open();
        assert!(settings.draft_gpui.log_drawer_open);
    }

    #[test]
    fn test_settings_save_clears_dirty_on_success() {
        let mut settings = SettingsView::new(ShellySettings::default(), GpuiUiConfig::default());
        settings.toggle_reduce_motion();
        assert!(settings.is_dirty);

        let result = settings.save_with(|_| Ok(()), |_| Ok(()));
        assert!(result.is_ok());
        assert!(!settings.is_dirty, "Successful save must clear dirty flag");
        assert!(settings.draft_gpui.reduce_motion);
    }

    #[test]
    fn test_settings_save_preserves_dirty_on_failure() {
        let mut settings = SettingsView::new(ShellySettings::default(), GpuiUiConfig::default());
        settings.toggle_aur();
        assert!(settings.is_dirty);

        let result = settings.save_with(|_| anyhow::bail!("Permission denied"), |_| Ok(()));
        assert!(result.is_err());
        assert!(
            settings.is_dirty,
            "Failed save must preserve dirty flag so draft is not lost"
        );
    }

    #[test]
    fn test_reduce_motion_toggle_updates_draft() {
        let mut settings = SettingsView::new(ShellySettings::default(), GpuiUiConfig::default());
        assert!(!settings.draft_gpui.reduce_motion);

        settings.toggle_reduce_motion();
        assert!(settings.draft_gpui.reduce_motion);
        assert!(settings.is_dirty);

        settings.toggle_reduce_motion();
        assert!(!settings.draft_gpui.reduce_motion);
    }

    #[test]
    fn test_settings_clean_state_and_draft_isolation() {
        let initial_shelly = ShellySettings::default();
        let initial_gpui = GpuiUiConfig::default();
        let mut settings = SettingsView::new(initial_shelly.clone(), initial_gpui.clone());
        assert!(!settings.is_dirty);

        // Toggling reduce motion makes it dirty
        settings.toggle_reduce_motion();
        assert!(settings.is_dirty);
        assert_ne!(
            settings.draft_gpui.reduce_motion,
            initial_gpui.reduce_motion
        );

        // Toggling back does not automatically clear is_dirty (draft was touched)
        settings.toggle_reduce_motion();
        assert!(settings.is_dirty);
        assert_eq!(
            settings.draft_gpui.reduce_motion,
            initial_gpui.reduce_motion
        );

        // Only explicit save clears is_dirty
        let result = settings.save_with(|_| Ok(()), |_| Ok(()));
        assert!(result.is_ok());
        assert!(!settings.is_dirty);
    }

    #[test]
    fn test_settings_reset_restores_saved_state() {
        let initial_shelly = ShellySettings::default();
        let initial_gpui = GpuiUiConfig::default();
        let mut settings = SettingsView::new(initial_shelly.clone(), initial_gpui.clone());

        settings.toggle_aur();
        settings.toggle_dark_theme();
        assert!(settings.is_dirty);

        settings.reset_to(initial_shelly.clone(), initial_gpui.clone());
        assert!(!settings.is_dirty);
        assert_eq!(settings.draft_shelly.aur_enabled, initial_shelly.aur_enabled);
        assert_eq!(settings.draft_gpui.dark_theme, initial_gpui.dark_theme);
    }
}
