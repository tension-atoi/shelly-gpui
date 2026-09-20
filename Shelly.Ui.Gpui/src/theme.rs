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
            bg_app: rgb(0x181825),
            bg_sidebar: rgb(0x11111b),
            bg_surface: rgb(0x1e1e2e),
            bg_surface_hover: rgb(0x28283d),
            bg_surface_active: rgb(0x313244),
            border: rgb(0x313244),
            border_focus: rgb(0x89b4fa),
            text_primary: rgb(0xcdd6f4),
            text_secondary: rgb(0xa6adc8),
            text_muted: rgb(0x6c7086),
            accent: rgb(0x89b4fa),
            accent_hover: rgb(0xb4befe),
            success: rgb(0xa6e3a1),
            warning: rgb(0xf9e2af),
            danger: rgb(0xf38ba8),
            badge_alpm: rgb(0x89b4fa),
            badge_aur: rgb(0xcba6f7),
            badge_flatpak: rgb(0x74c7ec),
            badge_appimage: rgb(0xfab387),
        }
    }

    pub fn light() -> Self {
        Self {
            bg_app: rgb(0xeff1f5),
            bg_sidebar: rgb(0xe6e9ef),
            bg_surface: rgb(0xffffff),
            bg_surface_hover: rgb(0xdce0e8),
            bg_surface_active: rgb(0xccd0da),
            border: rgb(0xbcc0cc),
            border_focus: rgb(0x1e66f5),
            text_primary: rgb(0x4c4f69),
            text_secondary: rgb(0x5c5f77),
            text_muted: rgb(0x8c8fa1),
            accent: rgb(0x1e66f5),
            accent_hover: rgb(0x04a5e5),
            success: rgb(0x40a02b),
            warning: rgb(0xdf8e1d),
            danger: rgb(0xd20f39),
            badge_alpm: rgb(0x1e66f5),
            badge_aur: rgb(0x8839ef),
            badge_flatpak: rgb(0x04a5e5),
            badge_appimage: rgb(0xfe640b),
        }
    }
}
