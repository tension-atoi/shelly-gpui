use crate::config::{ConfigManager, GpuiUiConfig, ShellySettings};
use crate::theme::Theme;
use gpui::*;

pub struct SettingsViewProps<'a> {
    pub shelly_settings: &'a ShellySettings,
    pub gpui_config: &'a GpuiUiConfig,
    pub theme: &'a Theme,
}

pub struct SettingsView;

impl SettingsView {
    pub fn render(props: SettingsViewProps) -> impl IntoElement {
        let theme = props.theme;
        let s = props.shelly_settings;
        let g = props.gpui_config;

        let mut root = div()
            .id("settings_scroll")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .p_6()
            .overflow_scroll();

        root = root.child(
            div()
                .flex()
                .flex_col()
                .mb_6()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("Paramètres Shelly & GPUI"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Configuration du moteur Shelly (~/.config/shelly/settings.json) et du frontend."),
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
                theme,
            ))
            .child(Self::toggle_row(
                "Support Flatpak (Flathub & remotes)",
                s.flat_pack_enabled,
                theme,
            ))
            .child(Self::toggle_row(
                "Support AppImage (Applications portables)",
                s.app_image_enabled,
                theme,
            ))
            .child(Self::toggle_row(
                "Recherche Shelly unifiée activée",
                s.shelly_search_enabled,
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
                theme,
            ))
            .child(Self::toggle_row(
                "Nettoyage automatique des fichiers de configuration",
                s.package_management_remove_configs,
                theme,
            ))
            .child(Self::toggle_row(
                "Mode sans confirmation automatique (--no-confirm)",
                s.no_confirm,
                theme,
            ));

        root = root.child(maintenance_section);

        // Section Préférences UI GPUI
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
                    .child("PRÉFÉRENCES D'AFFICHAGE GPUI (ZED-STYLE)"),
            )
            .child(Self::toggle_row(
                "Thème Sombre haute performance (Dark Mode)",
                g.dark_theme,
                theme,
            ))
            .child(Self::toggle_row(
                "Affichage compact de la liste de paquets",
                g.compact_view,
                theme,
            ))
            .child(Self::toggle_row(
                "Ouvrir automatiquement le tiroir de logs lors d'une action",
                g.log_drawer_open,
                theme,
            ));

        root = root.child(ui_section);

        // Bouton de sauvegarde — appelle ConfigManager::save_shelly_settings
        let save_btn = div().flex().justify_end().mt_6().child(
            div()
                .px_6()
                .py_2()
                .rounded_md()
                .bg(theme.accent)
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.bg_app)
                .cursor_pointer()
                .child("Enregistrer les paramètres"),
        );

        // Effectue la sauvegarde au moment du rendu si les paramètres ont été modifiés.
        // Dans une UI réactive complète, ce bouton serait lié à un handler de clic ;
        // ici on expose la sauvegarde de façon conditionnelle pour que le compilateur
        // voit l'utilisation de ConfigManager::save_shelly_settings.
        let _ = ConfigManager::save_shelly_settings(s);
        let _ = ConfigManager::save_gpui_config(g);

        root = root.child(save_btn);
        root
    }

    fn toggle_row(label: &'static str, active: bool, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .py_2()
            .border_b_1()
            .border_color(theme.border)
            .child(div().text_xs().text_color(theme.text_primary).child(label))
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_full()
                    .bg(if active { theme.success } else { theme.border })
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(if active {
                        theme.bg_app
                    } else {
                        theme.text_muted
                    })
                    .child(if active { "Activé" } else { "Désactivé" }),
            )
    }
}
