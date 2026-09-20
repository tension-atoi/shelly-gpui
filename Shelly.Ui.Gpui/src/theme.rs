use gpui::{rgb, Rgba};

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub bg_app: Rgba,
    pub bg_sidebar: Rgba,
    pub bg_surface: Rgba,
    pub bg_surface_hover: Rgba,
    pub bg_surface_active: Rgba,
    pub border: Rgba,
    pub border_focus: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub success: Rgba,
    pub warning: Rgba,
    pub danger: Rgba,
    pub danger_hover: Rgba,
    pub badge_alpm: Rgba,
    pub badge_aur: Rgba,
    pub badge_flatpak: Rgba,
    pub badge_appimage: Rgba,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg_app: rgb(0x0f0f14),            // Ultra-deep charcoal base
            bg_sidebar: rgb(0x0a0a0e),        // Dark contrast header/nav
            bg_surface: rgb(0x181822),        // Layered elevated card surface
            bg_surface_hover: rgb(0x232332),  // Immediate reactive hover state
            bg_surface_active: rgb(0x2c2d40), // Active item surface
            border: rgb(0x262638),            // Subtle 1px boundary frame
            border_focus: rgb(0x38bdf8),      // Electric cyan focus glow
            text_primary: rgb(0xf1f5f9),      // Crisp high-contrast slate-50
            text_secondary: rgb(0x94a3b8),    // Balanced legible secondary text
            text_muted: rgb(0x64748b),        // Subtle hints and captions
            accent: rgb(0x38bdf8),            // Electric cyan
            accent_hover: rgb(0x7dd3fc),      // Luminous cyan highlight
            success: rgb(0x10b981),           // Vibrant emerald
            warning: rgb(0xf59e0b),           // Bright amber
            danger: rgb(0xf43f5e),            // Vivid ruby rose
            danger_hover: rgb(0xfb7185),      // Luminous ruby highlight
            badge_alpm: rgb(0x38bdf8),        // Electric Cyan (Official ALPM)
            badge_aur: rgb(0xa855f7),         // Vivid Violet (AUR)
            badge_flatpak: rgb(0x06b6d4),     // Cyan (Flatpak)
            badge_appimage: rgb(0xf97316),    // Warm Orange (AppImage)
        }
    }

    pub fn light() -> Self {
        Self {
            bg_app: rgb(0xf8fafc),
            bg_sidebar: rgb(0xf1f5f9),
            bg_surface: rgb(0xffffff),
            bg_surface_hover: rgb(0xe2e8f0),
            bg_surface_active: rgb(0xcbd5e1),
            border: rgb(0xe2e8f0),
            border_focus: rgb(0x0284c7),
            text_primary: rgb(0x0f172a),
            text_secondary: rgb(0x475569),
            text_muted: rgb(0x94a3b8),
            accent: rgb(0x0284c7),
            accent_hover: rgb(0x0369a1),
            success: rgb(0x059669),
            warning: rgb(0xd97706),
            danger: rgb(0xe11d48),
            danger_hover: rgb(0xbe123c),
            badge_alpm: rgb(0x0284c7),
            badge_aur: rgb(0x7c3aed),
            badge_flatpak: rgb(0x0891b2),
            badge_appimage: rgb(0xea580c),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use super::*;

    #[test]
    fn test_theme_dark_and_light_parity_and_contrast() {
        let dark = Theme::dark();
        let light = Theme::light();

        // Ensure surface is distinct from app background in both themes
        assert_ne!(dark.bg_app, dark.bg_surface);
        assert_ne!(light.bg_app, light.bg_surface);

        // Ensure primary text differs from background in both themes
        assert_ne!(dark.text_primary, dark.bg_app);
        assert_ne!(light.text_primary, light.bg_app);

        // Ensure border focus differs from base border
        assert_ne!(dark.border, dark.border_focus);
        assert_ne!(light.border, light.border_focus);

        // Ensure default is dark theme
        let def = Theme::default();
        assert_eq!(def.bg_app, dark.bg_app);
        assert_eq!(def.accent, dark.accent);
    }
}

