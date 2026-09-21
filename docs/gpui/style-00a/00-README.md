# STYLE-00A — Visual Style Authority (Architecture Only)

## 0. Mission

STYLE-00A introduces a first-class **Visual Style authority** into Shelly GPUI
as pure architecture: typed style identities, a static profile registry, a
canonical surface-role vocabulary, a deterministic capability resolver, typed
persisted configuration, live CLI authority, and a Settings UI selector.

It produces **no visual change**: `Standard` reproduces the current stock UI
exactly, and `Transparency` is declared (effect requests + honest capability
reporting) without paint projection. Projection arrives with STYLE-00B, after
Render Lab stock evidence.

All STYLE-00A persistence is Shelly-local (`gpui-ui.json`). It is not a
gnos-ux, Gnosix, or `gnosis-shell-*` configuration authority, and it
pre-decides nothing about desktop-wide style persistence. Global
configuration authority remains OPEN (see `04-SETTINGS-AUTHORITY.md`).

## 1. Position in the Pipeline

```text
RENDER-00 (ratified, c31117c1)
    ↓
STYLE-00A (this phase — architecture review gate)
    ↓
RENDER-01 (stock capability evidence, evaluated under Standard)
    ↓
STYLE-00B (stock Transparency projection across real chrome)
    ↓
RENDER-02 / RENDER-03 (recipe + composition ceiling)
    ↓
RENDER-04 (renderer decision, only if warranted)
    ↓
STYLE-00C (proven richer effects)
```

## 2. Non-Goals (Hard)

- No third visual style; no speculative profile names.
- No GPUI fork, migration, WGSL, WGPU, shader ABI, or compositor protocol work.
- No paint/projection integration in shell components (STYLE-00B).
- No `render-lab style` axis (RENDER-01 pilot need; deferred, not forgotten).
- No quality axis, no parameter magnitudes (STYLE-00B with real consumers).
- No `Theme` struct expansion (colors stay the Theme authority).

## 3. Document Map

```text
docs/gpui/style-00a/
├── 00-README.md              # Mission, pipeline position, non-goals
├── 01-PROFILES-AND-ROLES.md  # Style IDs, schemes, roles, registry, effects
├── 02-WORK-CONTRACT.md       # Locked decisions, deviations, stop rule
├── 03-CLI-AND-STATUS.md      # appearance CLI surface and JSON schemas
├── 04-SETTINGS-AUTHORITY.md  # visual-style key, draft flow, dropdown UI
├── 05-ACCEPTANCE.md          # Gates matrix and closure evidence
└── evidence_style00a_settings_selector.png
```
