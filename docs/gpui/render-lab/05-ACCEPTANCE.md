# RENDER-00 Acceptance & Ratification Ledger

## 0. Baseline Reference

- Baseline Commit: `762471a5f980e57023f1de2e483adc5fb045b5b1`
- Phase Target: `RENDER-00`

---

## 1. Quality Gates & Test Verification

| Gate | Target | Result | Status |
|---|---|---|---|
| `cargo fmt --check` | 0 formatting diffs | Clean | PASS |
| `cargo clippy --release --locked -- -D warnings` | 0 warnings | Clean (1 future-incompat in third-party dep) | PASS |
| `cargo test --release --locked` | >= 142 tests passing | 142 passed, 0 failed | PASS |
| `cargo build --release --locked` | Clean release binary | Completed in 29.75s | PASS |

---

## 2. Invariant Sign-Off Matrix

| Invariant | Requirement | Verification Method | Status |
|---|---|---|---|
| **Corpus Completeness** | Exactly 46 fixtures (8G, 8H, 8I, 8J, 14K) | `test_fixture_catalog_completeness` | VERIFIED |
| **Integrity & Provenance** | Immutable source images hashed (SHA-256) | `SHA256SUMS` match against disk files | VERIFIED |
| **Capability Neutrality** | All 46 fixtures initialized to `UNKNOWN` | `test_fixture_catalog_completeness` | VERIFIED |
| **Deterministic State** | Frozen(t) clock mode and lab seeds | `test_clock_mode_serde` & catalog tests | VERIFIED |
| **Protocol v2 Authority** | Version 2 with 7 new render-lab commands | `test_render_lab_protocol_round_trip` | VERIFIED |
| **Sidebar Concealment** | RenderLab hidden from sidebar items | `components/sidebar.rs` destinations array | VERIFIED |
| **Session Memory Guard** | `set_destination` preserves last workspace | `test_set_destination_render_lab_does_not_update_last_workspace` | VERIFIED |
| **Zero Deadcode** | Deny dead code enforced | Strict rustc crate-root policy | VERIFIED |
| **Renderer Boundary** | Zero shader / custom WGSL / fork code | Architecture audit | VERIFIED |

---

## 3. Runtime & Packaging Provenance (Wayland Closure Evidence)

Verified live on 2026-09-21 against the exact installed build of `f673d9a2`.

| Check | Command / Probe | Result | Status |
|---|---|---|---|
| **Installed package** | `pacman -Q shelly-gpui-git` | `r4712.gf673d9a2-1` | VERIFIED |
| **Package ownership** | `pacman -Ql` | Owns `/usr/bin/shelly-gpui` and `/usr/lib/shelly/shelly-gpui-bin` | VERIFIED |
| **Canonical launch** | `shelly-gpui open` | PID `3615677` | VERIFIED |
| **PID provenance** | `readlink /proc/<pid>/exe` | `/usr/lib/shelly/shelly-gpui-bin` | VERIFIED |
| **Native Wayland** | `hyprctl clients` | `class: shelly-gpui`, `xwayland: 0` | VERIFIED |
| **Control protocol** | `shelly-gpui status` | Online, `Protocol: v2` | VERIFIED |
| **Offline manifest** | `render-lab status --json` (GUI offline) | `gui_running: false`, `catalog_count: 46` | VERIFIED |
| **Online manifest** | `render-lab status --json` (GUI online) | `gui_running: true`, `catalog_count: 46` | VERIFIED |
| **Fixture select** | `render-lab fixture depth.contact-shadow` | Active fixture round-trips in status + UI | VERIFIED |
| **Topology axis** | `render-lab topology full-band` | Reflected in status and ledger panel | VERIFIED |
| **Motion axis** | `render-lab motion smooth` | Reflected in status and ledger panel | VERIFIED |
| **Frozen clock** | `render-lab time 0.500` | `clock_mode: frozen`, `t = 0.500s` in ledger | VERIFIED |
| **Quality guard** | `render-lab quality ultra` | Clean rejection, exit 1, `stock`-only message | VERIFIED |
| **Nav transitions** | `navigate updates` / `render-lab open` / `navigate browse` | `updates` → `render lab` → `browse` | VERIFIED |
| **Sidebar concealment (runtime)** | Wayland screenshot | Rail shows Browse/Installed/Updates/News/Settings only | VERIFIED |
| **SHA-256 on disk** | `sha256sum -c SHA256SUMS` in `references/` | Both artifacts `OK` | VERIFIED |
| **SHA-256 in code** | `catalog.rs` constants vs `SHA256SUMS` | Exact match (GHIJ + K) | VERIFIED |
| **UI provenance** | Wayland screenshot, `depth.contact-shadow` | Board H (Cell 1), label, artifact, seed 2001, Deterministic | VERIFIED |
| **Capability neutrality (runtime)** | Wayland screenshot | `Capability: UNKNOWN` badge | VERIFIED |

Visual evidence: `evidence_render00_lab_populated.png` (cropped `shelly-gpui` window, 1263x1302, native Wayland).

### Non-blocking observation (candidate for RENDER-01)

- The full SHA-256 string is visually truncated in the provenance panel at this window width. The complete value remains available via manifest/JSON. Cosmetic only; no data loss.
