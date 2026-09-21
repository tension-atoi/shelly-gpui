# STYLE-00A Acceptance & Ratification Ledger

## 0. Baseline Reference

- Baseline Commit: `c31117c1e1eea68e427e08e5578987a80999e385` (RENDER-00 ratified)
- Phase Target: `STYLE-00A`

---

## 1. Quality Gates & Test Verification

| Gate | Target | Result | Status |
|---|---|---|---|
| `cargo fmt --check` | 0 formatting diffs | Clean | PASS |
| `cargo clippy --locked -- -D warnings` | 0 warnings | Clean (1 future-incompat in third-party dep) | PASS |
| `cargo test --release --locked` | 0 failures | 156 passed, 0 failed (142 baseline + 14 new) | PASS |
| `cargo build --release --locked` | Clean release binary | Completed | PASS |
| `zig build test` (Shelly.Cli.Zig) | 0 failures | 6 passed, 0 failed (backend untouched) | PASS |

New coverage: style/scheme/role serde round-trips, registry cardinality,
legacy + empty-config defaults, explicit-value preservation, deterministic
resolve over 13 × 2 × 2, Light/Dark effect parity, resolution honesty
(only `contact-depth` native, `ShaderRequired` unproducible), status schema,
CLI parse matrix (valid + 5 rejection cases), menu open/navigate/select,
draft dirty/save/reset with `visual_style`, settings completeness (14 keys).

---

## 2. Invariant Sign-Off Matrix

| Invariant | Requirement | Verification Method | Status |
|---|---|---|---|
| **Registry closure** | Exactly `Standard` + `Transparency` | `test_registry_contains_exactly_standard_and_transparency` | VERIFIED |
| **Role canon** | 13 roles, kebab stable, 8/5 text-bearing partition | role tests + `appearance resolve` | VERIFIED |
| **Config compatibility** | Missing field → `Standard`, legacy renders identical | serde tests + unchanged `Theme` | VERIFIED |
| **No Theme expansion** | `theme.rs` untouched | File ledger audit | VERIFIED |
| **Protocol stability** | Control protocol stays v2 | `protocol.rs` untouched, no new variants | VERIFIED |
| **Zero deadcode** | Deny policy enforced | Strict rustc crate-root policy + clippy | VERIFIED |
| **Renderer boundary** | No fork/WGSL/WGPU/compositor work | Architecture audit | VERIFIED |
| **No visual change** | Style switch alters no layout or paint | No paint consumer in 00A + runtime matrix | VERIFIED |
| **Read-your-writes** | `set` → `status` agreement, no restart | Runtime CLI matrix (online) | PENDING INSTALL |
| **Packaging provenance** | Exact-SHA `makepkg` + `pacman -U` + PID/Hyprland | Closure gate | PENDING INSTALL |

---

## 3. Runtime & Packaging Provenance (Wayland Closure Evidence)

To be completed after exact-SHA packaging and installation.
