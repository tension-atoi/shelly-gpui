# STYLE-00A Acceptance & Ratification Ledger

## 0. Baseline Reference

- Baseline Commit: `c31117c1e1eea68e427e08e5578987a80999e385` (RENDER-00 ratified)
- Phase Target: `STYLE-00A` + `STYLE-00A-R` final architecture closure

---

## 1. Quality Gates & Test Verification

| Gate | Target | Result | Status |
|---|---|---|---|
| `cargo fmt --check` | 0 formatting diffs | Clean | PASS |
| `cargo clippy --locked -- -D warnings` | 0 warnings | Clean (1 future-incompat in third-party dep) | PASS |
| `cargo test --release --locked` | 0 failures | 157 passed, 0 failed (142 baseline + 15 new incl. 00A-R) | PASS |
| `cargo build --release --locked` | Clean release binary | Completed | PASS |
| `zig build test` (Shelly.Cli.Zig) | 0 failures | 6 passed, 0 failed (backend untouched) | PASS |

New coverage: style/scheme/role serde round-trips, registry cardinality,
legacy + empty-config defaults, explicit-value preservation, deterministic
resolve over 13 × 2 × 2, Light/Dark effect parity, resolution neutrality
(all `UNKNOWN`, `ShaderRequired` unproducible), role→treatment mapping
(13/13), content-protection policy (12 protected / 1 unprotected), status
schema, CLI parse matrix (valid + 5 rejection cases), menu
open/navigate/select, draft dirty/save/reset with `visual_style`, settings
completeness (14 keys).

---

## 2. Invariant Sign-Off Matrix

| Invariant | Requirement | Verification Method | Status |
|---|---|---|---|
| **Registry closure** | Exactly `Standard` + `Transparency` | `test_registry_contains_exactly_standard_and_transparency` | VERIFIED |
| **Role canon** | 13 roles, kebab stable, 4 treatments, 12 protected / 1 unprotected | role tests + `appearance resolve` | VERIFIED |
| **Config compatibility** | Missing field → `Standard`, legacy renders identical | serde tests + unchanged `Theme` | VERIFIED |
| **No Theme expansion** | `theme.rs` untouched | File ledger audit | VERIFIED |
| **Protocol stability** | Control protocol stays v2 | `protocol.rs` untouched, no new variants | VERIFIED |
| **Zero deadcode** | Deny policy enforced | Strict rustc crate-root policy + clippy | VERIFIED |
| **Renderer boundary** | No fork/WGSL/WGPU/compositor work | Architecture audit | VERIFIED |
| **No visual change** | Style switch alters no layout or paint | No paint consumer in 00A + runtime matrix | VERIFIED |
| **Read-your-writes** | `set` → `status` agreement, no restart | Runtime CLI matrix (online) | VERIFIED |
| **Packaging provenance** | Exact-SHA `makepkg` + `pacman -U` + PID/Hyprland | Closure gate (`r4714.gdd2e4b55-1` pre-00A-R; 00A-R is architecture + docs, no repaint) | VERIFIED |
| **00A-R protection** | 12/13 roles protected; rail/query/inspector/console scrims | `test_content_protection_policy_covers_all_roles` + live `resolve` | VERIFIED |
| **00A-R treatment** | 13/13 roles mapped to 4 treatment classes | `test_surface_treatment_mapping_full_coverage` | VERIFIED |
| **00A-R neutrality** | All STYLE effects `UNKNOWN`, no `ShaderRequired` | `test_resolution_neutrality_all_unknown_in_style_00a` | VERIFIED |
| **00A-R boundary** | Shelly-local scope declared; global authority OPEN | `04-SETTINGS-AUTHORITY.md` §0 + `source` value | VERIFIED |

---

## 3. Runtime & Packaging Provenance (Wayland Closure Evidence)

Verified live on 2026-09-21 against the exact installed build of `dd2e4b55`.

| Check | Command / Probe | Result | Status |
|---|---|---|---|
| **Installed package** | `pacman -Q shelly-gpui-git` | `r4714.gdd2e4b55-1` | VERIFIED |
| **Exact-SHA build** | `makepkg` from pushed `main` (`dd2e4b55`) | `check()` 156 passed, 0 failed | VERIFIED |
| **Staging hygiene** | Build dir `/mnt/workbench/pkgbuild/style-00a` | No `/tmp` usage | VERIFIED |
| **Package ownership** | `pacman -Ql` | Owns `/usr/bin/shelly-gpui` and `/usr/lib/shelly/shelly-gpui-bin` | VERIFIED |
| **Canonical launch** | `shelly-gpui open` | PID `3843967` | VERIFIED |
| **PID provenance** | `readlink /proc/<pid>/exe` | `/usr/lib/shelly/shelly-gpui-bin` | VERIFIED |
| **Native Wayland** | `hyprctl clients` | `class: shelly-gpui`, `xwayland: 0` (tiled `1062x883`) | VERIFIED |
| **Read-your-writes** | `appearance style set standard` → `appearance status` | Reports `standard` immediately, no restart | VERIFIED |
| **Read-your-writes** | `appearance style set transparency` → `style get` | Reports `transparency` | VERIFIED |
| **Authority convergence** | `settings get visual-style` after `appearance` set | Agrees (`transparency`) | VERIFIED |
| **Protocol stability** | `status` on running GUI | `Protocol: v2` (unchanged) | VERIFIED |
| **Layout invariant** | Window geometry before/after style switch | Identical (`1062x883` at same origin) | VERIFIED |
| **Offline status** | `appearance status --json` (GUI offline) | Schema `shelly.appearance-status/1`, `committed-shelly-local-config` | VERIFIED |
| **Resolve matrix** | `appearance resolve` / `resolve menu` | 13 roles listed; `menu` → scrim yes, 4 effects | VERIFIED |
| **Rejections** | `style set neon`, `resolve glass-card` | Clean errors, exit `1` | VERIFIED |
| **Settings selector** | Wayland screenshot, committed config `transparency` | `Visual Style / Transparency ▾` row rendered in APPEARANCE section | VERIFIED |

Visual evidence: `evidence_style00a_settings_selector.png` (cropped
`shelly-gpui` window, native Wayland, code-identical build).

### Documented capture limitations (non-blocking)

- The dropdown-open state is proven by unit tests (`open`/`navigate`/
  `select`/dirty/save) over the shared, previously evidenced `MenuSurface`;
  no input-injection path was available for a live open-menu capture
  (`hyprctl dispatch` shim broken in this environment, no `ydotoold`
  daemon, no `wlrctl`/`wtype`) and none was started for evidence alone.
- An installed-binary re-capture was attempted and blocked by a parallel
  agent's overlay window; the committed capture comes from a code-identical
  build and the installed binary independently passed the full CLI matrix.
- Operator environment restored after verification (`visual-style`
  `standard`, window `1440.0x960.0`).
