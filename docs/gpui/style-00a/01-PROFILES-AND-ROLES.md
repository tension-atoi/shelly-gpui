# STYLE-00A Profiles, Roles & Registry

## 0. VisualStyleId

```rust
pub enum VisualStyleId { Standard, Transparency }
```

- Serde: stable kebab-case (`"standard"`, `"transparency"`).
- `Default` is `Standard`; unknown values are rejected, never coerced.
- Orthogonal to color scheme: `Dark + Transparency` is one style, not a profile.

## 1. ColorScheme

```rust
pub enum ColorScheme { Light, Dark }
```

Mapped from the persisted `dark_theme` boolean. `System` detection is
explicitly deferred: STYLE-00A introduces no platform theme probing.

## 2. SurfaceRole (13 Canonical Roles)

Components request a role; the style authority decides its projection.

| Role ID | Label | Treatment | Protected |
|---|---|---|---|
| `app-chrome` | App Chrome | ambient-chrome | no |
| `navigation-rail` | Navigation Rail | ambient-chrome | yes |
| `query-chrome` | Query Chrome | interactive-chrome | yes |
| `result-surface` | Result Surface | content-plane | yes |
| `inspector-chrome` | Inspector Chrome | interactive-chrome | yes |
| `inspector-content` | Inspector Content | content-plane | yes |
| `console-chrome` | Console Chrome | interactive-chrome | yes |
| `popover` | Popover | elevated-surface | yes |
| `menu` | Menu | elevated-surface | yes |
| `dialog` | Dialog | elevated-surface | yes |
| `control-surface` | Control Surface | interactive-chrome | yes |
| `content-surface` | Content Surface | content-plane | yes |
| `selection-surface` | Selection Surface | content-plane | yes |

Protected roles (`requires_content_protection`) own a predictable content
plane (scrim) under Transparency, so contrast is computed against a
style-owned background rather than an unknown wallpaper. Only `app-chrome`
— the base canvas owning no text of its own — resolves unprotected: rail
labels and counts, query text and filters, inspector metadata and tabs,
and console status and controls are all readable content in real Shelly
surfaces.

Treatment and protection are independent axes: `navigation-rail` is
`ambient-chrome` **and** protected. Treatment governs how strongly a style
may act; protection governs the contrast guarantee. Neither carries
visual magnitudes in STYLE-00A.

## 3. Registry

```rust
VisualStyleRegistry::get(VisualStyleId) -> &'static VisualStyleProfile
VisualStyleRegistry::all()              -> 2 profiles
```

Static Rust registry (no dynamic plugins). A profile provides a stable id,
a user-facing name, a revision (`1` in STYLE-00A), and declared semantic
effect requests.

## 4. Profiles

**Standard** (compatibility, default, fallback, regression baseline):

```text
requests: [contact-depth]  →  UNKNOWN (hypothesis only: stock path makes
Native plausible; RENDER-01 decides)
```

**Transparency** (coherent material system declaration):

```text
requests: [backdrop-blur, microstructure, rim-response, contact-depth]
```

## 5. EffectKind & Honest Resolution

Styles request intentions, never shaders. Resolution reuses the ratified
`render_lab::CapabilityClass` vocabulary (no duplicate enum):

| Effect | STYLE-00A verdict | Rationale |
|---|---|---|
| `contact-depth` | `UNKNOWN` | Hypothesis only: existing stock BoxShadow/border path makes Native plausible; RENDER-01 decides |
| `backdrop-blur` | `UNKNOWN` | Stock window-level path exists (`WindowBackgroundAppearance::Blurred` via `org_kde_kwin_blur`, compositor-dependent); per-surface evaluation deferred to RENDER-01 |
| `microstructure` | `UNKNOWN` | No proven stock texture path; spike scheduled in RENDER-01 |
| `rim-response` | `UNKNOWN` | Uniform borders proven; directional rim unevaluated until RENDER-01 |

STYLE-00A observes nothing: all four requests resolve `UNKNOWN`.
Existing stock support appears only as hypothesis notes. The first real
capability observations belong to the RENDER-01 backend-specific ledger,
so STYLE-00A never competes with it as a verdict authority.
`ShaderRequired` cannot be produced before RENDER-03. Every `UNKNOWN`
carries a machine-readable caveat note; nothing degrades silently.
