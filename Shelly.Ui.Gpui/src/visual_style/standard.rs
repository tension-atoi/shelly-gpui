use crate::visual_style::profile::{VisualStyleId, VisualStyleProfile};
use crate::visual_style::resolver::EffectKind;

/// Compatibility profile: the current non-transparent visual system.
///
/// Standard requests only [`EffectKind::ContactDepth`], resolved natively
/// by the existing stock UI. It is the default, the fallback, and the
/// regression baseline. STYLE-00A performs no Standard redesign.
pub static STANDARD_PROFILE: VisualStyleProfile = VisualStyleProfile {
    id: VisualStyleId::Standard,
    name: "Standard",
    revision: 1,
    requests: &[EffectKind::ContactDepth],
};
