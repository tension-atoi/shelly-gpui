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

| Role ID | Label | Text-bearing |
|---|---|---|
| `app-chrome` | App Chrome | no |
| `navigation-rail` | Navigation Rail | no |
| `query-chrome` | Query Chrome | no |
| `result-surface` | Result Surface | yes |
| `inspector-chrome` | Inspector Chrome | no |
| `inspector-content` | Inspector Content | yes |
| `console-chrome` | Console Chrome | no |
| `popover` | Popover | yes |
| `menu` | Menu | yes |
| `dialog` | Dialog | yes |
| `control-surface` | Control Surface | yes |
| `content-surface` | Content Surface | yes |
| `selection-surface` | Selection Surface | yes |

Text-bearing roles own a predictable content plane (scrim) under
Transparency, so contrast is computed against a style-owned background
rather than an unknown wallpaper. Pure chrome roles (5) carry no scrim.

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
requests: [contact-depth]  →  NATIVE (proven by the current stock UI)
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
| `contact-depth` | `NATIVE` | Borders, focus rings, shadows already render via public GPUI primitives |
| `backdrop-blur` | `UNKNOWN` | Stock window-level path exists (`WindowBackgroundAppearance::Blurred` via `org_kde_kwin_blur`, compositor-dependent); per-surface evaluation deferred to RENDER-01 |
| `microstructure` | `UNKNOWN` | No proven stock texture path; spike scheduled in RENDER-01 |
| `rim-response` | `UNKNOWN` | Uniform borders proven; directional rim unevaluated until RENDER-01 |

`ShaderRequired` cannot be produced before RENDER-03. Every `UNKNOWN`
carries a machine-readable caveat note; nothing degrades silently.
