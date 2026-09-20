use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Paramètres généraux Shelly partagés (~/.config/shelly/settings.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShellySettings {
    #[serde(default = "default_true")]
    pub aur_enabled: bool,
    #[serde(default = "default_true")]
    pub app_image_enabled: bool,
    #[serde(default = "default_true")]
    pub flat_pack_enabled: bool,
    #[serde(default = "default_true")]
    pub shelly_icons_enabled: bool,
    #[serde(default = "default_true")]
    pub shelly_search_enabled: bool,
    #[serde(default)]
    pub no_confirm: bool,
    #[serde(default)]
    pub package_management_cascade_delete: bool,
    #[serde(default)]
    pub package_management_remove_configs: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ShellySettings {
    fn default() -> Self {
        Self {
            aur_enabled: true,
            app_image_enabled: true,
            flat_pack_enabled: true,
            shelly_icons_enabled: true,
            shelly_search_enabled: true,
            no_confirm: true,
            package_management_cascade_delete: true,
            package_management_remove_configs: true,
        }
    }
}

/// Paramètres UI spécifiques à GPUI (~/.config/shelly/gpui-ui.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuiUiConfig {
    pub dark_theme: bool,
    pub window_width: f32,
    pub window_height: f32,
    pub compact_view: bool,
    pub log_drawer_open: bool,
    pub log_drawer_height: f32,
    pub last_selected_tab: usize,
}

impl Default for GpuiUiConfig {
    fn default() -> Self {
        Self {
            dark_theme: true,
            window_width: 1200.0,
            window_height: 820.0,
            compact_view: false,
            log_drawer_open: false,
            log_drawer_height: 220.0,
            last_selected_tab: 0,
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .map(|p| p.join("shelly"))
            .unwrap_or_else(|| PathBuf::from(".config/shelly"))
    }

    pub fn load_shelly_settings() -> ShellySettings {
        let path = Self::config_dir().join("settings.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<ShellySettings>(&content) {
                    return settings;
                }
            }
        }
        ShellySettings::default()
    }

    pub fn save_shelly_settings(settings: &ShellySettings) -> Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join("settings.json");
        let content = serde_json::to_string_pretty(settings)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn load_gpui_config() -> GpuiUiConfig {
        let path = Self::config_dir().join("gpui-ui.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<GpuiUiConfig>(&content) {
                    return config;
                }
            }
        }
        GpuiUiConfig::default()
    }

    pub fn save_gpui_config(config: &GpuiUiConfig) -> Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)?;
        let path = dir.join("gpui-ui.json");
        let content = serde_json::to_string_pretty(config)?;
        fs::write(path, content)?;
        Ok(())
    }
}
