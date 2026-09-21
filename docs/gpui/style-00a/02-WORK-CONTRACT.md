# STYLE-00A Work Contract

> **Baseline**: `c31117c1` (RENDER-00 ratified HEAD)
> **Status**: IMPLEMENTED — awaiting architecture review
> **Stop Rule**: STOP after STYLE-00A closure. RENDER-01 code waits for ratification.

## 0. Locked Decisions

1. **No protocol bump.** `appearance style set` reuses
   `ControlCommand::SettingsSet { key: "visual-style" }` online (inheriting
   the dirty-draft guard, atomic persist, runtime apply, and read-your-writes
   semantics) and `ConfigManager::set_setting` offline. `appearance status`
   and `appearance resolve` derive purely from committed config + the static
   registry, identically online and offline. Control protocol stays **v2**.
2. **No quality axis.** Multi-level degradation has no projecting consumer in
   00A; `Eco`/`Balanced`/`High` would violate strict YAGNI and `deny(dead_code)`.
   Deferred to STYLE-00B.
3. **No parameter magnitudes.** Transparency declares semantic `EffectKind`
   requests without strengths. `blur-strength`-style magnitudes arrive with
   STYLE-00B projection. No `appearance transparency set` command exists.
4. **Resolution vocabulary reused.** The resolver reports
   `render_lab::CapabilityClass` directly; no parallel `Native/Composed/
   Unsupported` enum is introduced.
5. **13 roles, all wired.** Every `SurfaceRole` is constructed via `ALL`,
   matched exhaustively by the resolver, reachable via
   `appearance resolve`, and covered by parameterized tests.
6. **Runtime authority is the config mirror.** `WorkspaceView.gpui_config.
   visual_style` is reloaded by `apply_committed_settings_to_runtime`; no
   `AppSession` field, no new invalidation channel. Zero visual change.
7. **Settings UI uses the ratified menu.** `MenuSurface` +
   `MenuCheckmarkItem` (exclusive selection), `deferred` + `anchored`
   dropdown, keyboard roving, focus rings — no cycle button.
8. **Honesty label in UI.** A static note under the selector states that
   Transparency projection arrives with STYLE-00B.

## 1. Deliberate Deviations from the Long Spec (§0–60)

| Spec suggestion | STYLE-00A choice | Rationale |
|---|---|---|
| `src/visual_style/` with `capability.rs`, `config.rs` | 5 files: `mod`, `profile`, `roles`, `resolver`, `standard`, `transparency` | Capability vocabulary reused from `render_lab`; config lives in the CONTROL-01 authority (`config.rs`) |
| Status exposes `quality` | Status exposes `visual_style`, `color_scheme`, `active_profile_revision`, `source`, `effects` | §40 schema marked "conceptual"; exact schema versioned at our discretion (`shelly.appearance-status/1`) |
| `render-lab style` axis | Deferred to RENDER-01 pilot | 00A scope is style authority, not lab consumption |
| Parameter schema with magnitudes | Requests-only declaration | "Only introduce fields that have real consumers" |

## 2. Adjacent Fix (Documented)

`--help` never listed the RENDER-00 `render-lab` commands. The help text now
documents both the `appearance` surface and the `render-lab` surface. No
behavior change.

## 3. File Ledger

```text
NEW  Shelly.Ui.Gpui/src/visual_style/mod.rs
NEW  Shelly.Ui.Gpui/src/visual_style/profile.rs
NEW  Shelly.Ui.Gpui/src/visual_style/roles.rs
NEW  Shelly.Ui.Gpui/src/visual_style/resolver.rs
NEW  Shelly.Ui.Gpui/src/visual_style/standard.rs
NEW  Shelly.Ui.Gpui/src/visual_style/transparency.rs
MOD  Shelly.Ui.Gpui/src/main.rs                      (module registration)
MOD  Shelly.Ui.Gpui/src/config.rs                    (visual_style + authority)
MOD  Shelly.Ui.Gpui/src/control/cli.rs               (appearance surface)
MOD  Shelly.Ui.Gpui/src/views/settings.rs            (dropdown selector)
MOD  Shelly.Ui.Gpui/src/views/workspace.rs           (handler wiring)
NEW  docs/gpui/style-00a/                            (this suite)
```

Untouched by construction: `theme.rs`, `control/protocol.rs` (v2 frozen),
all Zig backend sources, `render_lab/` sources.
