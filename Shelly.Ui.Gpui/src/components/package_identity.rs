use crate::backend::models::{UnifiedPackage, UnifiedPackageSource};
use crate::icons::AppIcon;
use crate::theme::Theme;
use gpui::*;
use std::path::PathBuf;

/// Identité visuelle résolue selon la chaîne déterministe à 3 tiers :
/// 1. Authentic: Icône authentique vérifiée sur le système de fichiers local
/// 2. Symbolic: Glyphe vectoriel symbolique certifié selon la source de paquets
/// 3. Fallback: Boîte générique neutre en cas de source inconnue
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedIdentity {
    Authentic(PathBuf),
    Symbolic(AppIcon),
    Fallback(AppIcon),
}

pub struct PackageIdentity;

impl PackageIdentity {
    /// Sonde le système de fichiers local pour localiser une icône authentique fournie par la source
    pub fn find_authentic_icon(pkg: &UnifiedPackage) -> Option<PathBuf> {
        match &pkg.inner {
            UnifiedPackageSource::AppImage(item) => {
                if let Some(ref icon_name) = item.icon_name {
                    let p = PathBuf::from(icon_name);
                    if p.is_file() {
                        return Some(p);
                    }
                    if let Some(found) = Self::probe_icon_name(icon_name) {
                        return Some(found);
                    }
                }
                if let Some(ref path_str) = item.path {
                    let p = PathBuf::from(path_str);
                    let sibling_png = p.with_extension("png");
                    if sibling_png.is_file() {
                        return Some(sibling_png);
                    }
                }
                None
            }
            UnifiedPackageSource::Flatpak(hit) => {
                if let Some(ref app_id) = hit.app_id {
                    Self::probe_flatpak_icon(app_id)
                } else {
                    None
                }
            }
            UnifiedPackageSource::Standard(alpm) => Self::probe_desktop_icon(&alpm.name),
            UnifiedPackageSource::Aur(aur) => Self::probe_desktop_icon(&aur.name),
        }
    }

    fn probe_flatpak_icon(app_id: &str) -> Option<PathBuf> {
        let mut roots = vec![PathBuf::from(
            "/var/lib/flatpak/exports/share/icons/hicolor",
        )];
        if let Some(d) = dirs::data_dir() {
            roots.push(d.join("flatpak/exports/share/icons/hicolor"));
        }
        let resolutions = ["128x128", "scalable", "64x64", "48x48", "32x32"];
        for root in &roots {
            if !root.exists() {
                continue;
            }
            for res in &resolutions {
                let png = root.join(res).join("apps").join(format!("{}.png", app_id));
                if png.is_file() {
                    return Some(png);
                }
                let svg = root.join(res).join("apps").join(format!("{}.svg", app_id));
                if svg.is_file() {
                    return Some(svg);
                }
            }
        }
        None
    }

    fn probe_desktop_icon(name: &str) -> Option<PathBuf> {
        let pixmap_png = PathBuf::from(format!("/usr/share/pixmaps/{}.png", name));
        if pixmap_png.is_file() {
            return Some(pixmap_png);
        }
        let pixmap_svg = PathBuf::from(format!("/usr/share/pixmaps/{}.svg", name));
        if pixmap_svg.is_file() {
            return Some(pixmap_svg);
        }
        let hicolor_root = PathBuf::from("/usr/share/icons/hicolor");
        if hicolor_root.exists() {
            for res in &["128x128", "scalable", "64x64", "48x48", "32x32"] {
                let png = hicolor_root
                    .join(res)
                    .join("apps")
                    .join(format!("{}.png", name));
                if png.is_file() {
                    return Some(png);
                }
                let svg = hicolor_root
                    .join(res)
                    .join("apps")
                    .join(format!("{}.svg", name));
                if svg.is_file() {
                    return Some(svg);
                }
            }
        }
        None
    }

    fn probe_icon_name(icon_name: &str) -> Option<PathBuf> {
        if let Some(found) = Self::probe_desktop_icon(icon_name) {
            return Some(found);
        }
        if let Some(data_dir) = dirs::data_dir() {
            let user_icons = data_dir.join("icons/hicolor");
            if user_icons.exists() {
                for res in &["128x128", "scalable", "64x64", "48x48", "32x32"] {
                    let png = user_icons
                        .join(res)
                        .join("apps")
                        .join(format!("{}.png", icon_name));
                    if png.is_file() {
                        return Some(png);
                    }
                    let svg = user_icons
                        .join(res)
                        .join("apps")
                        .join(format!("{}.svg", icon_name));
                    if svg.is_file() {
                        return Some(svg);
                    }
                }
            }
        }
        None
    }

    /// Résout l'identité visuelle selon la chaîne de vérité déterministe à 3 tiers
    pub fn resolve_identity(pkg: &UnifiedPackage) -> ResolvedIdentity {
        // Tier 1: Icône authentique fournie par la source vérifiée localement
        if let Some(path) = Self::find_authentic_icon(pkg) {
            return ResolvedIdentity::Authentic(path);
        }

        // Tier 2: Icône symbolique vectorielle vérifiée selon la source
        match pkg.source_type.to_uppercase().as_str() {
            "ALPM" => ResolvedIdentity::Symbolic(AppIcon::SourceAlpm),
            "AUR" => ResolvedIdentity::Symbolic(AppIcon::SourceAur),
            "FLATPAK" => ResolvedIdentity::Symbolic(AppIcon::SourceFlatpak),
            "APPIMAGE" => ResolvedIdentity::Symbolic(AppIcon::SourceAppImage),
            // Tier 3: Repli générique
            _ => ResolvedIdentity::Fallback(AppIcon::PackageGeneric),
        }
    }

    /// Couleur d'accentuation sémantique de la source
    pub fn source_color(source_type: &str, theme: &Theme) -> Rgba {
        match source_type.to_uppercase().as_str() {
            "ALPM" => theme.badge_alpm,
            "AUR" => theme.badge_aur,
            "FLATPAK" => theme.badge_flatpak,
            "APPIMAGE" => theme.badge_appimage,
            _ => theme.text_muted,
        }
    }

    /// Teinte d'arrière-plan de l'avatar selon la source
    pub fn avatar_bg_color(source_type: &str, theme: &Theme) -> Rgba {
        let base = Self::source_color(source_type, theme);
        Rgba {
            r: base.r,
            g: base.g,
            b: base.b,
            a: 0.12,
        }
    }

    /// Teinte de bordure subtile de l'avatar selon la source
    pub fn avatar_border_color(source_type: &str, theme: &Theme) -> Rgba {
        let base = Self::source_color(source_type, theme);
        Rgba {
            r: base.r,
            g: base.g,
            b: base.b,
            a: 0.28,
        }
    }

    /// Rendu de l'avatar d'identité dans un conteneur géométrique stable (pour les Cartes)
    pub fn render_avatar(
        pkg: &UnifiedPackage,
        size: f32,
        corner_radius: f32,
        theme: &Theme,
    ) -> impl IntoElement {
        let identity = Self::resolve_identity(pkg);
        let color = Self::source_color(&pkg.source_type, theme);
        let bg = Self::avatar_bg_color(&pkg.source_type, theme);
        let border = Self::avatar_border_color(&pkg.source_type, theme);
        let glyph_size = (size * 0.56).round();

        let mut container = div()
            .size(px(size))
            .flex_shrink_0()
            .rounded(px(corner_radius))
            .bg(bg)
            .border_1()
            .border_color(border)
            .flex()
            .items_center()
            .justify_center();

        match identity {
            ResolvedIdentity::Authentic(path) => {
                if path.extension().is_some_and(|ext| ext == "svg") {
                    container = container.child(
                        svg()
                            .path(path.to_string_lossy().to_string())
                            .size(px(glyph_size)),
                    );
                } else {
                    container = container.child(
                        img(path)
                            .size(px(glyph_size))
                            .object_fit(ObjectFit::Contain),
                    );
                }
            }
            ResolvedIdentity::Symbolic(icon) | ResolvedIdentity::Fallback(icon) => {
                container = container.child(
                    svg()
                        .path(icon.path())
                        .size(px(glyph_size))
                        .text_color(color),
                );
            }
        }

        container
    }

    /// Rendu d'un glyphe d'identité compact en ligne (pour le Tableau)
    pub fn render_inline_glyph(pkg: &UnifiedPackage, size: f32, theme: &Theme) -> impl IntoElement {
        let identity = Self::resolve_identity(pkg);
        let color = Self::source_color(&pkg.source_type, theme);

        let mut container = div()
            .size(px(size))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center();

        match identity {
            ResolvedIdentity::Authentic(path) => {
                if path.extension().is_some_and(|ext| ext == "svg") {
                    container = container.child(
                        svg()
                            .path(path.to_string_lossy().to_string())
                            .size(px(size)),
                    );
                } else {
                    container =
                        container.child(img(path).size(px(size)).object_fit(ObjectFit::Contain));
                }
            }
            ResolvedIdentity::Symbolic(icon) | ResolvedIdentity::Fallback(icon) => {
                container =
                    container.child(svg().path(icon.path()).size(px(size)).text_color(color));
            }
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::models::{
        AlpmPackage, AppImageItem, AurPackage, FlatpakHit, UnifiedPackageSource,
    };
    use core::prelude::v1::test;

    #[test]
    fn test_resolve_identity_3_tier_chain() {
        // Tier 1: Authentic icon from existing file on disk
        let mut temp_icon = std::env::temp_dir();
        temp_icon.push("shelly_test_authentic_icon.png");
        std::fs::write(&temp_icon, b"dummy png content").unwrap();

        let appimage_with_icon = UnifiedPackage {
            name: "CustomApp".into(),
            version: "1.0.0".into(),
            description: "test".into(),
            source_type: "AppImage".into(),
            repository_or_remote: "appimage".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::AppImage(AppImageItem {
                icon_name: Some(temp_icon.to_string_lossy().to_string()),
                ..Default::default()
            }),
        };
        assert_eq!(
            PackageIdentity::resolve_identity(&appimage_with_icon),
            ResolvedIdentity::Authentic(temp_icon.clone())
        );
        let _ = std::fs::remove_file(temp_icon);

        // Tier 2: Verified source symbolic icon (when no authentic file exists)
        let alpm = UnifiedPackage {
            name: "ripgrep-cli-tool-without-desktop-icon".into(),
            version: "14.1.0".into(),
            description: "fast search".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_identity(&alpm),
            ResolvedIdentity::Symbolic(AppIcon::SourceAlpm)
        );

        let aur = UnifiedPackage {
            name: "nonexistent-aur-cli-package".into(),
            version: "1.0".into(),
            description: "cli".into(),
            source_type: "AUR".into(),
            repository_or_remote: "aur".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Aur(AurPackage::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_identity(&aur),
            ResolvedIdentity::Symbolic(AppIcon::SourceAur)
        );

        let flatpak = UnifiedPackage {
            name: "org.nonexistent.FlatpakApp".into(),
            version: "1.0".into(),
            description: "".into(),
            source_type: "Flatpak".into(),
            repository_or_remote: "flathub".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Flatpak(FlatpakHit::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_identity(&flatpak),
            ResolvedIdentity::Symbolic(AppIcon::SourceFlatpak)
        );

        // Tier 3: Generic fallback box for unknown source
        let unknown = UnifiedPackage {
            name: "custom-tool".into(),
            version: "0.1".into(),
            description: "".into(),
            source_type: "CustomRepo".into(),
            repository_or_remote: "custom".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_identity(&unknown),
            ResolvedIdentity::Fallback(AppIcon::PackageGeneric)
        );
    }

    #[test]
    fn test_source_colors_and_alpha_invariants() {
        let theme = Theme::dark();
        for source in &["ALPM", "AUR", "Flatpak", "AppImage"] {
            let color = PackageIdentity::source_color(source, &theme);
            let bg = PackageIdentity::avatar_bg_color(source, &theme);
            let border = PackageIdentity::avatar_border_color(source, &theme);

            assert_eq!(bg.r, color.r);
            assert_eq!(bg.g, color.g);
            assert_eq!(bg.b, color.b);
            assert!((bg.a - 0.12).abs() < 0.001);

            assert_eq!(border.r, color.r);
            assert_eq!(border.g, color.g);
            assert_eq!(border.b, color.b);
            assert!((border.a - 0.28).abs() < 0.001);
        }
    }
}
