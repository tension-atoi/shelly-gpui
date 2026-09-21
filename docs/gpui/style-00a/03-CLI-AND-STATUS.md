# STYLE-00A CLI & Status Reference

## 0. Command Surface

```bash
shelly-gpui appearance style get
shelly-gpui appearance style set standard|transparency
shelly-gpui appearance status [--json]
shelly-gpui appearance resolve [role] [--json]
```

- `style get` reads the committed `visual-style` key (online and offline).
- `style set` writes through the CONTROL-01 settings authority: live socket
  dispatch with dirty-draft protection when the GUI runs, direct atomic
  config write when offline. Invalid values exit `1` at parse time.
- `status` reports the committed style, scheme, profile revision, source,
  and per-effect resolutions. No socket required; deterministic.
- `resolve` without an argument lists the 13 canonical roles; with a
  kebab-case role it prints the resolved `StyleProjection`. Unknown roles
  exit `1` with the valid list.

The same key is reachable through the generic authority:

```bash
shelly-gpui settings get visual-style
shelly-gpui settings set visual-style transparency
shelly-gpui settings reset visual-style
```

## 1. Status Schema (`shelly.appearance-status/1`)

```json
{
  "schema": "shelly.appearance-status/1",
  "visual_style": "transparency",
  "color_scheme": "dark",
  "active_profile_revision": 1,
  "source": "committed-shelly-local-config",
  "effects": [
    {
      "kind": "backdrop-blur",
      "requested": true,
      "resolved": "UNKNOWN",
      "note": "Stock window-level path exists via WindowBackgroundAppearance::Blurred (org_kde_kwin_blur, compositor-dependent); per-surface evaluation deferred to RENDER-01"
    }
  ]
}
```

`source: "committed-shelly-local-config"` states the authority actually
read: Shelly's committed on-disk configuration, never an unsaved GUI draft
and never a global desktop configuration authority (which remains OPEN).

## 2. Projection Schema (resolve)

```json
{
  "style": "transparency",
  "role": "menu",
  "treatment": "elevated-surface",
  "color_scheme": "dark",
  "opaque": false,
  "content_scrim": true,
  "effects": ["…same EffectResolution objects…"]
}
```

Resolution is a pure function of `(style, role, scheme)`: no I/O, no clock,
no randomness. All 13 × 2 × 2 combinations are asserted in unit tests.
