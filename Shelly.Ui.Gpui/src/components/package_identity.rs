use crate::backend::models::{UnifiedPackage, UnifiedPackageSource};
use crate::icons::AppIcon;
use crate::state::PackageKey;
use crate::theme::Theme;
use gpui::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

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

static IDENTITY_CACHE: OnceLock<RwLock<HashMap<PackageKey, ResolvedIdentity>>> = OnceLock::new();
static RESOLVING_KEYS: OnceLock<RwLock<HashSet<PackageKey>>> = OnceLock::new();

fn get_identity_cache() -> &'static RwLock<HashMap<PackageKey, ResolvedIdentity>> {
    IDENTITY_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn get_resolving_keys() -> &'static RwLock<HashSet<PackageKey>> {
    RESOLVING_KEYS.get_or_init(|| RwLock::new(HashSet::new()))
}

pub struct PackageIdentity;

impl PackageIdentity {
    /// Sonde le système de fichiers local pour localiser une icône authentique fournie par la source.
    /// Pour ALPM et AUR, inspecte les fichiers possédés par le paquet dans /var/lib/pacman/local/<pkg>-*/files.
    pub fn find_authentic_icon(pkg: &UnifiedPackage) -> Option<PathBuf> {
        match &pkg.inner {
            UnifiedPackageSource::AppImage(item) => {
                if let Some(ref icon_name) = item.icon_name {
                    let p = PathBuf::from(icon_name);
                    if p.is_file() {
                        return Some(p);
                    }
                    if let Some(found) = Self::resolve_icon_name(icon_name) {
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
            UnifiedPackageSource::Standard(_) | UnifiedPackageSource::Aur(_) => {
                Self::probe_alpm_or_aur_icon(pkg)
            }
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
            if !root.is_dir() {
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

    /// Résolution authentique basée sur la provenance stricte pour ALPM et AUR.
    /// Inspecte la base locale pacman (/var/lib/pacman/local/<pkg>-<version>/files) pour identifier
    /// les fichiers .desktop réellement possédés par le paquet.
    fn probe_alpm_or_aur_icon(pkg: &UnifiedPackage) -> Option<PathBuf> {
        if !pkg.is_installed {
            return None;
        }
        let pacman_local = Path::new("/var/lib/pacman/local");
        if !pacman_local.is_dir() {
            return None;
        }

        let prefix = format!("{}-", pkg.name);
        let read_dir = std::fs::read_dir(pacman_local).ok()?;
        let mut desktop_rel_path: Option<String> = None;

        for entry in read_dir.flatten() {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if let Some(rest) = name_str.strip_prefix(&prefix) {
                if rest.starts_with(|c: char| c.is_ascii_digit()) || rest.contains(':') {
                    let files_path = entry.path().join("files");
                    if let Ok(content) = std::fs::read_to_string(&files_path) {
                        let mut fallback_desktop = None;
                        for line in content.lines() {
                            let line = line.trim();
                            if line.starts_with("usr/share/applications/")
                                && line.ends_with(".desktop")
                            {
                                if line.ends_with(&format!("{}.desktop", pkg.name)) {
                                    desktop_rel_path = Some(line.to_string());
                                    break;
                                } else if fallback_desktop.is_none() {
                                    fallback_desktop = Some(line.to_string());
                                }
                            }
                        }
                        if desktop_rel_path.is_none() {
                            desktop_rel_path = fallback_desktop;
                        }
                    }
                    if desktop_rel_path.is_some() {
                        break;
                    }
                }
            }
        }

        let desktop_rel = desktop_rel_path?;
        let full_desktop_path = PathBuf::from("/").join(desktop_rel);
        if !full_desktop_path.is_file() {
            return None;
        }

        let icon_val = Self::parse_desktop_icon_field(&full_desktop_path)?;
        Self::resolve_icon_name(&icon_val)
    }

    /// Extrait le champ Icon= de la section [Desktop Entry] d'un fichier .desktop
    fn parse_desktop_icon_field(desktop_path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(desktop_path).ok()?;
        let mut in_desktop_entry = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            } else if trimmed.starts_with('[') && in_desktop_entry {
                break;
            }
            if in_desktop_entry && trimmed.starts_with("Icon=") {
                let val = trimmed.strip_prefix("Icon=").unwrap_or("").trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
        None
    }

    /// Résout un nom d'icône ou chemin absolu sur le système de fichiers standard
    fn resolve_icon_name(icon_name: &str) -> Option<PathBuf> {
        let p = PathBuf::from(icon_name);
        if p.is_file() {
            return Some(p);
        }

        let extensions: &[&str] = if icon_name.ends_with(".png") || icon_name.ends_with(".svg") {
            &[""]
        } else {
            &[".png", ".svg"]
        };

        // 1. /usr/share/pixmaps
        for ext in extensions {
            let pix = PathBuf::from(format!("/usr/share/pixmaps/{}{}", icon_name, ext));
            if pix.is_file() {
                return Some(pix);
            }
        }

        // 2. /usr/share/icons/hicolor
        let hicolor_root = PathBuf::from("/usr/share/icons/hicolor");
        if hicolor_root.is_dir() {
            for res in &["128x128", "scalable", "64x64", "48x48", "256x256", "32x32"] {
                for ext in extensions {
                    let path = hicolor_root
                        .join(res)
                        .join("apps")
                        .join(format!("{}{}", icon_name, ext));
                    if path.is_file() {
                        return Some(path);
                    }
                }
            }
        }

        // 3. Répertoire utilisateur (~/.local/share/icons/hicolor)
        if let Some(data_dir) = dirs::data_dir() {
            let user_icons = data_dir.join("icons/hicolor");
            if user_icons.is_dir() {
                for res in &["128x128", "scalable", "64x64", "48x48", "256x256", "32x32"] {
                    for ext in extensions {
                        let path = user_icons
                            .join(res)
                            .join("apps")
                            .join(format!("{}{}", icon_name, ext));
                        if path.is_file() {
                            return Some(path);
                        }
                    }
                }
            }
        }

        None
    }

    /// Dérive l'identité de repli symbolique ou générique en O(1) sans accès I/O
    pub fn symbolic_or_fallback(pkg: &UnifiedPackage) -> ResolvedIdentity {
        match pkg.source_type.to_uppercase().as_str() {
            "ALPM" => ResolvedIdentity::Symbolic(AppIcon::SourceAlpm),
            "AUR" => ResolvedIdentity::Symbolic(AppIcon::SourceAur),
            "FLATPAK" => ResolvedIdentity::Symbolic(AppIcon::SourceFlatpak),
            "APPIMAGE" => ResolvedIdentity::Symbolic(AppIcon::SourceAppImage),
            _ => ResolvedIdentity::Fallback(AppIcon::PackageGeneric),
        }
    }

    /// Résout l'identité visuelle de façon déterministe et synchrone
    pub fn resolve_identity_sync(pkg: &UnifiedPackage) -> ResolvedIdentity {
        if let Some(path) = Self::find_authentic_icon(pkg) {
            ResolvedIdentity::Authentic(path)
        } else {
            Self::symbolic_or_fallback(pkg)
        }
    }

    /// Résout l'identité visuelle de manière non-bloquante pour le chemin chaud de rendu GPUI.
    ///
    /// - Cache HIT: Retourne immédiatement le résultat en O(1) mémoire sans aucun I/O disque.
    /// - Cache MISS: Retourne immédiatement l'icône symbolique/fallback O(1) et planifie
    ///   une résolution asynchrone sur un thread d'arrière-plan pour peupler le cache.
    pub fn resolve_identity(pkg: &UnifiedPackage) -> ResolvedIdentity {
        let key = pkg.key();

        if let Ok(cache) = get_identity_cache().read() {
            if let Some(resolved) = cache.get(&key) {
                return resolved.clone();
            }
        }

        let fallback = Self::symbolic_or_fallback(pkg);

        let should_spawn = if let Ok(mut pending) = get_resolving_keys().write() {
            pending.insert(key.clone())
        } else {
            false
        };

        if should_spawn {
            let pkg_clone = pkg.clone();
            let key_clone = key.clone();
            std::thread::spawn(move || {
                let resolved = Self::resolve_identity_sync(&pkg_clone);
                if let Ok(mut cache) = get_identity_cache().write() {
                    cache.insert(key_clone.clone(), resolved);
                }
                if let Ok(mut pending) = get_resolving_keys().write() {
                    pending.remove(&key_clone);
                }
            });
        }

        fallback
    }

    /// Précharge en arrière-plan les identités d'une collection de paquets
    pub fn preload(packages: &[UnifiedPackage]) {
        let mut to_resolve = Vec::new();
        if let Ok(cache) = get_identity_cache().read() {
            for pkg in packages {
                let key = pkg.key();
                if !cache.contains_key(&key) {
                    to_resolve.push(pkg.clone());
                }
            }
        }
        if to_resolve.is_empty() {
            return;
        }

        std::thread::spawn(move || {
            for pkg in to_resolve {
                let key = pkg.key();
                let resolved = Self::resolve_identity_sync(&pkg);
                if let Ok(mut cache) = get_identity_cache().write() {
                    cache.insert(key, resolved);
                }
            }
        });
    }

    #[cfg(test)]
    pub fn clear_cache() {
        if let Ok(mut cache) = get_identity_cache().write() {
            cache.clear();
        }
        if let Ok(mut pending) = get_resolving_keys().write() {
            pending.clear();
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
        PackageIdentity::clear_cache();

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
            PackageIdentity::resolve_identity_sync(&appimage_with_icon),
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
            PackageIdentity::resolve_identity_sync(&alpm),
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
            PackageIdentity::resolve_identity_sync(&aur),
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
            PackageIdentity::resolve_identity_sync(&flatpak),
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
            PackageIdentity::resolve_identity_sync(&unknown),
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

    #[test]
    fn test_identity_cache_zero_io_and_preload() {
        PackageIdentity::clear_cache();

        let pkg = UnifiedPackage {
            name: "cache-test-pkg".into(),
            version: "1.0.0".into(),
            description: "testing cache".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        // Cache miss: returns immediate symbolic fallback without blocking
        let initial = PackageIdentity::resolve_identity(&pkg);
        assert_eq!(initial, ResolvedIdentity::Symbolic(AppIcon::SourceAlpm));

        // Preload inserts into cache
        PackageIdentity::preload(&[pkg.clone()]);
        // Give background thread a tiny moment to write
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Cache hit: returns cached identity
        let cached = PackageIdentity::resolve_identity(&pkg);
        assert_eq!(cached, ResolvedIdentity::Symbolic(AppIcon::SourceAlpm));

        PackageIdentity::clear_cache();
    }

    #[test]
    fn test_alpm_uninstalled_never_checks_disk() {
        PackageIdentity::clear_cache();

        // Even with a name matching a real app like "firefox", if uninstalled it must be Symbolic
        let uninstalled_pkg = UnifiedPackage {
            name: "firefox".into(),
            version: "128.0".into(),
            description: "browser".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        assert_eq!(PackageIdentity::find_authentic_icon(&uninstalled_pkg), None);
        assert_eq!(
            PackageIdentity::resolve_identity_sync(&uninstalled_pkg),
            ResolvedIdentity::Symbolic(AppIcon::SourceAlpm)
        );
    }
}
