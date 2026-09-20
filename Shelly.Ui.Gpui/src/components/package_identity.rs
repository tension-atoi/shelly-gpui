use crate::backend::models::UnifiedPackage;
use crate::icons::AppIcon;
use crate::theme::Theme;
use gpui::*;

pub struct PackageIdentity;

impl PackageIdentity {
    /// Résout l'icône canonique selon la chaîne de vérité stricte :
    /// 1. Icône authentique fournie par la source (ex. AppImage si valide)
    /// 2. Icône symbolique vectorielle vérifiée selon la source (ALPM -> Arch, AUR -> AUR, Flatpak -> Cube, AppImage -> Diamond)
    /// 3. Repli générique (boîte de paquet neutre)
    ///
    /// Règle absolue : ZÉRO fausses icônes / logos devinés par heuristique.
    pub fn resolve_icon(pkg: &UnifiedPackage) -> AppIcon {
        match pkg.source_type.to_uppercase().as_str() {
            "ALPM" => AppIcon::SourceAlpm,
            "AUR" => AppIcon::SourceAur,
            "FLATPAK" => AppIcon::SourceFlatpak,
            "APPIMAGE" => AppIcon::SourceAppImage,
            _ => AppIcon::PackageGeneric,
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
        let icon = Self::resolve_icon(pkg);
        let color = Self::source_color(&pkg.source_type, theme);
        let bg = Self::avatar_bg_color(&pkg.source_type, theme);
        let border = Self::avatar_border_color(&pkg.source_type, theme);
        let glyph_size = (size * 0.56).round();

        div()
            .size(px(size))
            .flex_shrink_0()
            .rounded(px(corner_radius))
            .bg(bg)
            .border_1()
            .border_color(border)
            .flex()
            .items_center()
            .justify_center()
            .child(
                svg()
                    .path(icon.path())
                    .size(px(glyph_size))
                    .text_color(color),
            )
    }

    /// Rendu d'un glyphe d'identité compact en ligne (pour le Tableau)
    pub fn render_inline_glyph(pkg: &UnifiedPackage, size: f32, theme: &Theme) -> impl IntoElement {
        let icon = Self::resolve_icon(pkg);
        let color = Self::source_color(&pkg.source_type, theme);

        div()
            .size(px(size))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center()
            .child(svg().path(icon.path()).size(px(size)).text_color(color))
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
    fn test_resolve_icon_deterministic_source_mapping() {
        let alpm = UnifiedPackage {
            name: "ripgrep".into(),
            version: "14.1.0".into(),
            description: "fast search".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };
        assert_eq!(PackageIdentity::resolve_icon(&alpm), AppIcon::SourceAlpm);

        let aur = UnifiedPackage {
            name: "visual-studio-code-bin".into(),
            version: "1.92.0".into(),
            description: "editor".into(),
            source_type: "AUR".into(),
            repository_or_remote: "aur".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Aur(AurPackage::default()),
        };
        assert_eq!(PackageIdentity::resolve_icon(&aur), AppIcon::SourceAur);

        let flatpak = UnifiedPackage {
            name: "org.mozilla.firefox".into(),
            version: "128.0".into(),
            description: "browser".into(),
            source_type: "Flatpak".into(),
            repository_or_remote: "flathub".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Flatpak(FlatpakHit::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_icon(&flatpak),
            AppIcon::SourceFlatpak
        );

        let appimage = UnifiedPackage {
            name: "Obsidian".into(),
            version: "1.6.5".into(),
            description: "knowledge base".into(),
            source_type: "AppImage".into(),
            repository_or_remote: "appimage".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::AppImage(AppImageItem::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_icon(&appimage),
            AppIcon::SourceAppImage
        );

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
            PackageIdentity::resolve_icon(&unknown),
            AppIcon::PackageGeneric
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
