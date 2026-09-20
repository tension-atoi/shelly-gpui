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
    pub on_toggle_shelly_search: WindowActionHandler,
    pub on_toggle_cascade_delete: WindowActionHandler,
    pub on_toggle_remove_configs: WindowActionHandler,
    pub on_toggle_no_confirm: WindowActionHandler,
    pub on_toggle_dark_theme: WindowActionHandler,
    pub on_toggle_compact_view: WindowActionHandler,
    pub on_toggle_log_drawer_auto_open: WindowActionHandler,
    pub on_toggle_reduce_motion: WindowActionHandler,
    pub on_save: WindowActionHandler,
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

    pub fn toggle_shelly_search(&mut self) {
        self.draft_shelly.shelly_search_enabled = !self.draft_shelly.shelly_search_enabled;
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

    pub fn toggle_no_confirm(&mut self) {
        self.draft_shelly.no_confirm = !self.draft_shelly.no_confirm;
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
                                .child("Paramètres Shelly & GPUI"),
                        )
                        .child(if is_dirty {
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme.warning)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.bg_app)
                                .child("Modifications non enregistrées")
                        } else {
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme.border)
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child("À jour")
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Configuration du moteur Shelly (~/.config/shelly/settings.json) et des interactions GPUI."),
                ),
        );

        // Section Sources de paquets
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
                    .child("SOURCES DE PAQUETS"),
            )
            .child(Self::toggle_row(
                "Support AUR (Arch User Repository)",
                s.aur_enabled,
                props.on_toggle_aur,
                theme,
            ))
            .child(Self::toggle_row(
                "Support Flatpak (Flathub & remotes)",
                s.flat_pack_enabled,
                props.on_toggle_flatpak,
                theme,
            ))
            .child(Self::toggle_row(
                "Support AppImage (Applications portables)",
                s.app_image_enabled,
                props.on_toggle_appimage,
                theme,
            ))
            .child(Self::toggle_row(
                "Recherche Shelly unifiée activée",
                s.shelly_search_enabled,
                props.on_toggle_shelly_search,
                theme,
            ));

        root = root.child(sources_section);

        // Section Maintenance & Sécurité
        let maintenance_section = div()
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
                    .child("MAINTENANCE ET SUPPRESSION"),
            )
            .child(Self::toggle_row(
                "Suppression en cascade des dépendances orphelines",
                s.package_management_cascade_delete,
                props.on_toggle_cascade_delete,
                theme,
            ))
            .child(Self::toggle_row(
                "Nettoyage automatique des fichiers de configuration",
                s.package_management_remove_configs,
                props.on_toggle_remove_configs,
                theme,
            ))
            .child(Self::toggle_row(
                "Mode sans confirmation automatique (--no-confirm)",
                s.no_confirm,
                props.on_toggle_no_confirm,
                theme,
            ));

        root = root.child(maintenance_section);

        // Section Préférences UI & Mouvement GPUI
        let ui_section = div()
            .flex()
            .flex_col()
            .p_4()
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
                    .child("PRÉFÉRENCES D'AFFICHAGE & MOUVEMENT GPUI"),
            )
            .child(Self::toggle_row(
                "Thème Sombre haute performance (Dark Mode)",
                g.dark_theme,
                props.on_toggle_dark_theme,
                theme,
            ))
            .child(Self::toggle_row(
                "Affichage compact de la liste de paquets",
                g.compact_view,
                props.on_toggle_compact_view,
                theme,
            ))
            .child(Self::toggle_row(
                "Ouvrir automatiquement le tiroir de logs lors d'une action",
                g.log_drawer_open,
                props.on_toggle_log_drawer_auto_open,
                theme,
            ))
            .child(Self::toggle_row(
                "Réduire les animations (Reduce motion — transitions instantanées)",
                g.reduce_motion,
                props.on_toggle_reduce_motion,
                theme,
            ));

        root = root.child(ui_section);

        // Bouton de sauvegarde explicite
        let on_save = props.on_save;
        let save_btn = div().flex().justify_end().mt_6().child(
            div()
                .id("save_settings_btn")
                .px_6()
                .py_2()
                .rounded_md()
                .bg(if is_dirty {
                    theme.accent
                } else {
                    theme.bg_surface_active
                })
                .border_1()
                .border_color(if is_dirty { theme.accent } else { theme.border })
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(if is_dirty {
                    theme.bg_app
                } else {
                    theme.text_secondary
                })
                .cursor_pointer()
                .hover(move |s| s.bg(theme.accent_hover).text_color(theme.bg_app))
                .child(if is_dirty {
                    "Enregistrer les paramètres *"
                } else {
                    "Paramètres enregistrés"
                })
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    on_save(window, cx);
                }),
        );

        root = root.child(save_btn);
        root
    }

    fn toggle_row(
        label: &'static str,
        active: bool,
        on_toggle: WindowActionHandler,
        theme: &Theme,
    ) -> impl IntoElement {
        let hover_bg = theme.bg_surface_hover;
        div()
            .flex()
            .items_center()
            .justify_between()
            .py_2()
            .px_2()
            .rounded_sm()
            .border_b_1()
            .border_color(theme.border)
            .hover(move |s| s.bg(hover_bg))
            .child(div().text_xs().text_color(theme.text_primary).child(label))
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
                    .child(if active { "Activé" } else { "Désactivé" })
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

        settings.toggle_shelly_search();
        assert!(!settings.draft_shelly.shelly_search_enabled);

        settings.toggle_cascade_delete();
        assert!(!settings.draft_shelly.package_management_cascade_delete);

        settings.toggle_remove_configs();
        assert!(!settings.draft_shelly.package_management_remove_configs);

        settings.toggle_no_confirm();
        assert!(!settings.draft_shelly.no_confirm);

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
}
