use crate::render_lab::CapabilityClass;
use crate::visual_style::profile::{ColorScheme, VisualStyleId, VisualStyleRegistry};
use crate::visual_style::roles::SurfaceRole;
use serde::{Deserialize, Serialize};

/// Semantic effect requested by a visual style profile.
///
/// A style never names a concrete shader or backend primitive. It requests
/// an intention; the resolver reports how (and whether) the stock backend
/// can satisfy it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffectKind {
    BackdropBlur,
    Microstructure,
    RimResponse,
    ContactDepth,
}

impl EffectKind {
    pub const ALL: &[EffectKind] = &[
        EffectKind::BackdropBlur,
        EffectKind::Microstructure,
        EffectKind::RimResponse,
        EffectKind::ContactDepth,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackdropBlur => "backdrop-blur",
            Self::Microstructure => "microstructure",
            Self::RimResponse => "rim-response",
            Self::ContactDepth => "contact-depth",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BackdropBlur => "Backdrop Blur",
            Self::Microstructure => "Microstructure",
            Self::RimResponse => "Rim Response",
            Self::ContactDepth => "Contact Depth",
        }
    }
}

impl std::fmt::Display for EffectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Honest per-effect capability report for the stock GPUI backend.
///
/// STYLE-00A proves nothing new: only [`EffectKind::ContactDepth`] resolves
/// to [`CapabilityClass::Native`], by existence of the current stock UI
/// (borders, shadows). Every other request stays [`CapabilityClass::Unknown`]
/// with a caveat documenting the known stock path or the missing evidence.
/// No request may resolve to `ShaderRequired` before RENDER-03.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectResolution {
    pub kind: EffectKind,
    pub requested: bool,
    pub resolved: CapabilityClass,
    pub note: &'static str,
}

impl EffectResolution {
    pub fn resolve(kind: EffectKind) -> Self {
        let (resolved, note) = match kind {
            EffectKind::BackdropBlur => (
                CapabilityClass::Unknown,
                "Stock window-level path exists via WindowBackgroundAppearance::Blurred (org_kde_kwin_blur, compositor-dependent); per-surface evaluation deferred to RENDER-01",
            ),
            EffectKind::Microstructure => (
                CapabilityClass::Unknown,
                "No proven stock texture path in STYLE-00A; spike scheduled in RENDER-01",
            ),
            EffectKind::RimResponse => (
                CapabilityClass::Unknown,
                "Uniform borders proven in stock UI; directional rim unevaluated until RENDER-01",
            ),
            EffectKind::ContactDepth => (
                CapabilityClass::Native,
                "Proven by the current stock UI: borders, focus rings and drop shadows render via public GPUI primitives",
            ),
        };
        Self {
            kind,
            requested: true,
            resolved,
            note,
        }
    }
}

/// Resolved visual projection for one (style, role, scheme) triple.
///
/// STYLE-00A projections are reporting-consumed (CLI status/resolve):
/// paint integration arrives with STYLE-00B.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StyleProjection {
    pub style: VisualStyleId,
    pub role: SurfaceRole,
    pub color_scheme: ColorScheme,
    pub opaque: bool,
    pub content_scrim: bool,
    pub effects: Vec<EffectResolution>,
}

/// Machine-readable appearance authority report.
///
/// `source` names the authority actually read: the committed on-disk
/// configuration, never an unsaved GUI draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppearanceStatus {
    pub schema: &'static str,
    pub visual_style: VisualStyleId,
    pub color_scheme: ColorScheme,
    pub active_profile_revision: u32,
    pub source: &'static str,
    pub effects: Vec<EffectResolution>,
}

pub const APPEARANCE_STATUS_SCHEMA: &str = "shelly.appearance-status/1";

/// Deterministically resolves a style projection for one surface role.
///
/// Given identical (style, role, scheme), the output is identical.
/// Resolution performs no I/O.
pub fn resolve_style(
    style: VisualStyleId,
    role: SurfaceRole,
    color_scheme: ColorScheme,
) -> StyleProjection {
    let profile = VisualStyleRegistry::get(style);
    let effects: Vec<EffectResolution> = profile
        .requests
        .iter()
        .copied()
        .map(EffectResolution::resolve)
        .collect();
    let opaque = style == VisualStyleId::Standard;
    let content_scrim = style == VisualStyleId::Transparency && role.is_text_bearing();
    StyleProjection {
        style,
        role,
        color_scheme,
        opaque,
        content_scrim,
        effects,
    }
}

/// Builds the appearance status report from committed configuration values.
pub fn status_for_config(visual_style: VisualStyleId, dark_theme: bool) -> AppearanceStatus {
    let profile = VisualStyleRegistry::get(visual_style);
    AppearanceStatus {
        schema: APPEARANCE_STATUS_SCHEMA,
        visual_style,
        color_scheme: ColorScheme::from_dark_theme(dark_theme),
        active_profile_revision: profile.revision,
        source: "committed-config",
        effects: profile
            .requests
            .iter()
            .copied()
            .map(EffectResolution::resolve)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_effect_kind_kebab_serde_and_labels() {
        assert_eq!(EffectKind::ALL.len(), 4);
        for kind in EffectKind::ALL {
            let json = serde_json::to_string(kind).expect("Serialize effect kind");
            assert_eq!(json, format!("\"{}\"", kind.as_str()));
            let parsed: EffectKind = serde_json::from_str(&json).expect("Deserialize effect kind");
            assert_eq!(*kind, parsed);
            assert_eq!(kind.to_string(), kind.as_str());
        }
        assert_eq!(EffectKind::BackdropBlur.label(), "Backdrop Blur");
        assert_eq!(EffectKind::ContactDepth.label(), "Contact Depth");
    }

    #[test]
    fn test_resolution_honesty_only_contact_depth_is_native() {
        for kind in EffectKind::ALL {
            let res = EffectResolution::resolve(*kind);
            assert!(res.requested);
            assert!(!res.note.is_empty());
            match kind {
                EffectKind::ContactDepth => assert_eq!(res.resolved, CapabilityClass::Native),
                _ => assert_eq!(
                    res.resolved,
                    CapabilityClass::Unknown,
                    "{kind} must stay Unknown until RENDER-01 evidence"
                ),
            }
            assert_ne!(
                res.resolved,
                CapabilityClass::ShaderRequired,
                "ShaderRequired is forbidden before RENDER-03"
            );
        }
    }

    #[test]
    fn test_resolve_deterministic_across_all_roles_styles_schemes() {
        for style in VisualStyleId::ALL {
            for role in SurfaceRole::ALL {
                for scheme in ColorScheme::ALL {
                    let first = resolve_style(*style, *role, *scheme);
                    let second = resolve_style(*style, *role, *scheme);
                    assert_eq!(first, second, "Resolution must be deterministic");
                    assert_eq!(first.style, *style);
                    assert_eq!(first.role, *role);
                    assert_eq!(first.color_scheme, *scheme);
                    assert_eq!(first.opaque, *style == VisualStyleId::Standard);
                    assert_eq!(
                        first.content_scrim,
                        *style == VisualStyleId::Transparency && role.is_text_bearing()
                    );
                    let expected_requests = VisualStyleRegistry::get(*style).requests.len();
                    assert_eq!(first.effects.len(), expected_requests);
                }
            }
        }
    }

    #[test]
    fn test_resolve_light_dark_parity_same_effects() {
        for style in VisualStyleId::ALL {
            for role in SurfaceRole::ALL {
                let light = resolve_style(*style, *role, ColorScheme::Light);
                let dark = resolve_style(*style, *role, ColorScheme::Dark);
                assert_eq!(light.effects, dark.effects);
                assert_eq!(light.opaque, dark.opaque);
                assert_eq!(light.content_scrim, dark.content_scrim);
            }
        }
    }

    #[test]
    fn test_appearance_status_schema_and_serialization() {
        let status = status_for_config(VisualStyleId::Transparency, true);
        assert_eq!(status.schema, APPEARANCE_STATUS_SCHEMA);
        assert_eq!(status.visual_style, VisualStyleId::Transparency);
        assert_eq!(status.color_scheme, ColorScheme::Dark);
        assert_eq!(status.active_profile_revision, 1);
        assert_eq!(status.source, "committed-config");
        assert_eq!(status.effects.len(), 4);
        let json = serde_json::to_string(&status).expect("Serialize status");
        assert!(json.contains("\"transparency\""));
        assert!(json.contains(APPEARANCE_STATUS_SCHEMA));
        assert!(json.contains("\"committed-config\""));

        let standard = status_for_config(VisualStyleId::Standard, false);
        assert_eq!(standard.color_scheme, ColorScheme::Light);
        assert_eq!(standard.effects.len(), 1);
        assert_eq!(standard.effects[0].kind, EffectKind::ContactDepth);
        assert_eq!(standard.effects[0].resolved, CapabilityClass::Native);
    }
}
