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
    #[serde(default = "default_dark_theme")]
    pub dark_theme: bool,
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    #[serde(default = "default_compact_view")]
    pub compact_view: bool,
    #[serde(default = "default_log_drawer_open")]
    pub log_drawer_open: bool,
    #[serde(default = "default_log_drawer_height")]
    pub log_drawer_height: f32,
    #[serde(default = "default_last_selected_tab")]
    pub last_selected_tab: usize,
    #[serde(default)]
    pub reduce_motion: bool,
}

pub const MIN_WINDOW_WIDTH: f32 = 1024.0;
pub const MIN_WINDOW_HEIGHT: f32 = 680.0;

fn default_dark_theme() -> bool {
    true
}

fn default_window_width() -> f32 {
    1280.0
}

fn default_window_height() -> f32 {
    840.0
}

fn default_compact_view() -> bool {
    false
}

fn default_log_drawer_open() -> bool {
    false
}

fn default_log_drawer_height() -> f32 {
    220.0
}

fn default_last_selected_tab() -> usize {
    0
}

impl Default for GpuiUiConfig {
    fn default() -> Self {
        Self {
            dark_theme: default_dark_theme(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            compact_view: default_compact_view(),
            log_drawer_open: default_log_drawer_open(),
            log_drawer_height: default_log_drawer_height(),
            last_selected_tab: default_last_selected_tab(),
            reduce_motion: false,
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

    pub fn sanitize_window_size(w: f32, h: f32) -> (f32, f32) {
        let width = if w.is_nan() || w < MIN_WINDOW_WIDTH {
            default_window_width()
        } else {
            w
        };
        let height = if h.is_nan() || h < MIN_WINDOW_HEIGHT {
            default_window_height()
        } else {
            h
        };
        (width, height)
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

    pub fn load_gpui_config_sanitized() -> GpuiUiConfig {
        let mut config = Self::load_gpui_config();
        let (w, h) = Self::sanitize_window_size(config.window_width, config.window_height);
        config.window_width = w;
        config.window_height = h;
        config
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

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_gpui_config_all_fields_default_on_empty_json() {
        let empty_json = "{}";
        let parsed: GpuiUiConfig = serde_json::from_str(empty_json)
            .expect("Empty JSON object should deserialize with default values for all fields");

        assert!(parsed.dark_theme);
        assert_eq!(parsed.window_width, 1280.0);
        assert_eq!(parsed.window_height, 840.0);
        assert!(!parsed.compact_view);
        assert!(!parsed.log_drawer_open);
        assert_eq!(parsed.log_drawer_height, 220.0);
        assert_eq!(parsed.last_selected_tab, 0);
        assert!(!parsed.reduce_motion);
    }

    #[test]
    fn test_sanitize_window_size_enforces_minimums() {
        // Below minimum falls back to default
        let (w, h) = ConfigManager::sanitize_window_size(800.0, 500.0);
        assert_eq!(w, 1280.0);
        assert_eq!(h, 840.0);

        // NaN falls back to default
        let (w_nan, h_nan) = ConfigManager::sanitize_window_size(f32::NAN, f32::NAN);
        assert_eq!(w_nan, 1280.0);
        assert_eq!(h_nan, 840.0);

        // Valid size >= minimum is preserved
        let (w_valid, h_valid) = ConfigManager::sanitize_window_size(1600.0, 1000.0);
        assert_eq!(w_valid, 1600.0);
        assert_eq!(h_valid, 1000.0);

        // Boundary test at exact minimums
        let (w_min, h_min) = ConfigManager::sanitize_window_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);
        assert_eq!(w_min, 1024.0);
        assert_eq!(h_min, 680.0);
    }

    #[test]
    fn test_gpui_config_backward_compatibility_without_reduce_motion() {
        let legacy_json = r#"{
            "dark_theme": true,
            "window_width": 1280.0,
            "window_height": 840.0,
            "compact_view": true,
            "log_drawer_open": true,
            "log_drawer_height": 250.0,
            "last_selected_tab": 2
        }"#;

        let parsed: GpuiUiConfig = serde_json::from_str(legacy_json)
            .expect("Legacy config without reduce_motion should deserialize successfully");

        assert!(parsed.dark_theme);
        assert_eq!(parsed.window_width, 1280.0);
        assert_eq!(parsed.window_height, 840.0);
        assert!(parsed.compact_view);
        assert!(parsed.log_drawer_open);
        assert_eq!(parsed.log_drawer_height, 250.0);
        assert_eq!(parsed.last_selected_tab, 2);
        assert!(
            !parsed.reduce_motion,
            "Default for reduce_motion must be false"
        );
    }

    #[test]
    fn test_gpui_config_with_explicit_reduce_motion() {
        let json_with_reduce = r#"{
            "dark_theme": false,
            "window_width": 1024.0,
            "window_height": 768.0,
            "compact_view": false,
            "log_drawer_open": false,
            "log_drawer_height": 200.0,
            "last_selected_tab": 0,
            "reduce_motion": true
        }"#;

        let parsed: GpuiUiConfig = serde_json::from_str(json_with_reduce)
            .expect("Config with reduce_motion should deserialize successfully");

        assert!(parsed.reduce_motion);
    }
}
