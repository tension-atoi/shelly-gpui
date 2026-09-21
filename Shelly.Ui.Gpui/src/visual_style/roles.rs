use serde::{Deserialize, Serialize};

/// Semantic surface role authority.
///
/// Components request a role; the style authority decides its visual
/// projection. No widget may hardcode transparency, blur, or shader
/// selection for itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceRole {
    AppChrome,
    NavigationRail,
    QueryChrome,
    ResultSurface,
    InspectorChrome,
    InspectorContent,
    ConsoleChrome,
    Popover,
    Menu,
    Dialog,
    ControlSurface,
    ContentSurface,
    SelectionSurface,
}

impl SurfaceRole {
    pub const ALL: &[SurfaceRole] = &[
        SurfaceRole::AppChrome,
        SurfaceRole::NavigationRail,
        SurfaceRole::QueryChrome,
        SurfaceRole::ResultSurface,
        SurfaceRole::InspectorChrome,
        SurfaceRole::InspectorContent,
        SurfaceRole::ConsoleChrome,
        SurfaceRole::Popover,
        SurfaceRole::Menu,
        SurfaceRole::Dialog,
        SurfaceRole::ControlSurface,
        SurfaceRole::ContentSurface,
        SurfaceRole::SelectionSurface,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppChrome => "app-chrome",
            Self::NavigationRail => "navigation-rail",
            Self::QueryChrome => "query-chrome",
            Self::ResultSurface => "result-surface",
            Self::InspectorChrome => "inspector-chrome",
            Self::InspectorContent => "inspector-content",
            Self::ConsoleChrome => "console-chrome",
            Self::Popover => "popover",
            Self::Menu => "menu",
            Self::Dialog => "dialog",
            Self::ControlSurface => "control-surface",
            Self::ContentSurface => "content-surface",
            Self::SelectionSurface => "selection-surface",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::AppChrome => "App Chrome",
            Self::NavigationRail => "Navigation Rail",
            Self::QueryChrome => "Query Chrome",
            Self::ResultSurface => "Result Surface",
            Self::InspectorChrome => "Inspector Chrome",
            Self::InspectorContent => "Inspector Content",
            Self::ConsoleChrome => "Console Chrome",
            Self::Popover => "Popover",
            Self::Menu => "Menu",
            Self::Dialog => "Dialog",
            Self::ControlSurface => "Control Surface",
            Self::ContentSurface => "Content Surface",
            Self::SelectionSurface => "Selection Surface",
        }
    }

    /// Roles bearing text or data own a predictable content plane under
    /// Transparency, so contrast is computed against a style-owned
    /// background rather than an unknown wallpaper.
    pub fn is_text_bearing(self) -> bool {
        match self {
            Self::AppChrome
            | Self::NavigationRail
            | Self::QueryChrome
            | Self::InspectorChrome
            | Self::ConsoleChrome => false,
            Self::ResultSurface
            | Self::InspectorContent
            | Self::Popover
            | Self::Menu
            | Self::Dialog
            | Self::ControlSurface
            | Self::ContentSurface
            | Self::SelectionSurface => true,
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        for role in Self::ALL {
            if role.as_str() == value.to_ascii_lowercase() {
                return Ok(*role);
            }
        }
        let valid: Vec<&str> = Self::ALL.iter().map(|r| r.as_str()).collect();
        Err(format!(
            "Invalid surface role '{value}', expected one of: {}",
            valid.join(", ")
        ))
    }
}

impl std::fmt::Display for SurfaceRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_surface_role_count_and_kebab_serde() {
        assert_eq!(SurfaceRole::ALL.len(), 13);
        for role in SurfaceRole::ALL {
            let json = serde_json::to_string(role).expect("Serialize role");
            assert_eq!(json, format!("\"{}\"", role.as_str()));
            let parsed: SurfaceRole = serde_json::from_str(&json).expect("Deserialize role");
            assert_eq!(*role, parsed);
            assert_eq!(role.to_string(), role.as_str());
            assert_eq!(
                *role,
                SurfaceRole::parse(role.as_str()).expect("parse role")
            );
        }
        assert!(SurfaceRole::parse("glass-card").is_err());
        assert!(SurfaceRole::parse("").is_err());
    }

    #[test]
    fn test_text_bearing_partition_covers_all_roles() {
        let bearing: Vec<SurfaceRole> = SurfaceRole::ALL
            .iter()
            .copied()
            .filter(|r| r.is_text_bearing())
            .collect();
        let chrome: Vec<SurfaceRole> = SurfaceRole::ALL
            .iter()
            .copied()
            .filter(|r| !r.is_text_bearing())
            .collect();
        assert_eq!(bearing.len(), 8);
        assert_eq!(chrome.len(), 5);
        assert_eq!(bearing.len() + chrome.len(), SurfaceRole::ALL.len());
        assert!(SurfaceRole::ContentSurface.is_text_bearing());
        assert!(SurfaceRole::Menu.is_text_bearing());
        assert!(!SurfaceRole::AppChrome.is_text_bearing());
        assert!(!SurfaceRole::NavigationRail.is_text_bearing());
    }
}
