# RENDER Acceptance & Ratification Ledger

## 0. Baseline References & Commit Provenance

- **RENDER-00 Baseline Commit**: `762471a5f980e57023f1de2e483adc5fb045b5b1`
- **Canonical Re-entry Anchor**: `0f319c7aab800c9d1254ef6f98c223215b051083`
- **RENDER-01 Implementation Commit**: `d79a9e68e4bf9c05da124804300438cf1a2f64fa`
- **Target Slice**: `RENDER-01 (Confrontation & Capability Ledger Closure)`
- **Platform**: Arch Linux (Kernel 6.18.2-arch1-1), Hyprland Wayland (Display `wayland-1`), NVIDIA RTX 3070 (Driver 570.86.16)

---

## 1. Quality Gates & Test Verification

| Gate | Target | Result | Status |
|---|---|---|---|
| `cargo fmt --check` | 0 formatting diffs | Clean | **PASS** |
| `cargo clippy --release --locked -- -D warnings` | 0 warnings | Clean (zero warnings tolerated) | **PASS** |
| `cargo test --release --locked` | >= 173 tests passing | 173 passed, 0 failed in 0.05s | **PASS** |
| `cargo build --release --locked` | Clean release binary | Completed cleanly | **PASS** |
| `zig build test` (all Zig components) | All backend tests passing | 100% passed across all crates | **PASS** |
| `CapabilityLedger::validate()` | 46 validated observations | 46 valid, 0 unknown, 0 shader-required | **PASS** |

---

## 2. Invariant Sign-Off Matrix

| Invariant | Requirement | Verification Method | Status |
|---|---|---|---|
| **Corpus Completeness** | Exactly 46 fixtures (8G, 8H, 8I, 8J, 14K) | `test_fixture_catalog_completeness` + confrontation registry | **VERIFIED** |
| **Integrity & Provenance** | Immutable source reference images hashed (SHA-256) | `SHA256SUMS` match against disk files | **VERIFIED** |
| **Empirical Verdicts** | All 46 fixtures evaluated; 0 UNKNOWN remaining | `CapabilityLedger::validate()` & `EVIDENCE_INDEX.json` | **VERIFIED** |
| **Zero Speculative Shaders** | `SHADER_REQUIRED` strictly forbidden in RENDER-01 | Ledger validation enforcing `verdict != ShaderRequired` | **VERIFIED** |
| **Deterministic RGBA Hashes** | Dual-pass screenshot capture with identical ROI hashes | 46 `.roi.png` and `.manifest.json` with SHA-256 | **VERIFIED** |
| **Deterministic Textures** | 17 procedural texture kinds via `SplitMix64` | `texture.rs` deterministic test suite | **VERIFIED** |
| **ASTRA Replay & Qualification**| 8/8 ASTRA-FORGE-00 stages replayed & qualified | Complete forensic ingest; qualified as non-canonical reference | **VERIFIED** |
| **Renderer Boundary** | Stock GPUI 0.2.2 public primitives only | Architecture audit: zero custom shaders or forks in Shelly | **VERIFIED** |
| **Zero Deadcode Policy** | Strict crate-root deny (`dead_code`, `unused_*`) | Cargo clippy and rustc compiler enforcement | **VERIFIED** |
| **Sidebar Concealment** | RenderLab hidden from public navigation rail | `components/sidebar.rs` destination filter | **VERIFIED** |
| **Session Memory Guard** | Navigation preserves active workspace memory | `test_set_destination_render_lab_does_not_update_last_workspace` | **VERIFIED** |

---

## 3. Evidence Corpus Machine Audit & Semantic Verdicts

Automated machine audit executed via `docs/gpui/render-lab/tools/audit_evidence.py` directly against the decoded RGBA buffers of all `.roi.png` files and manifests:

```text
canonical catalog fixtures       46
ledger observations              46
evidence manifests               46
ROI images                       46

missing fixture IDs               0
duplicate fixture IDs             0
unknown fixture IDs               0

runs_identical=false              0
missing pixel hashes              0
missing recipes                   0
missing backend IDs               0
decoded RGBA hash mismatches      0

semantic verdict mismatches       0
```

### Family Breakdown:
- **Family G (Fields)** (8/8 OK): 2 NATIVE, 6 COMPOSABLE, 0 TEXTURE_PROOF
- **Family H (Depth & Elevation)** (8/8 OK): 4 NATIVE, 4 COMPOSABLE, 0 TEXTURE_PROOF
- **Family I (Optical & Glow)** (8/8 OK): 2 NATIVE, 6 COMPOSABLE, 0 TEXTURE_PROOF
- **Family J (Tactile Substrates)** (8/8 OK): 4 NATIVE, 2 COMPOSABLE, 2 TEXTURE_PROOF
- **Family K (Noise & Microtextures)** (14/14 OK): 0 NATIVE, 0 COMPOSABLE, 14 TEXTURE_PROOF

### Candidate Capability Ledger Breakdown:

| Capability Class | Count | Percentage | Description |
|---|---|---|---|
| **`NATIVE`** | 12 | 26.1% | Directly reproducible at full design fidelity via stock GPUI 0.2.2 primitives (`generated_texture == false`) |
| **`COMPOSABLE`** | 18 | 39.1% | Assembled from multi-layer native primitives, borders, clipping, and shadows (`generated_texture == false`) |
| **`TEXTURE_PROOF`** | 16 | 34.8% | Pre-rendered or procedurally synthesized deterministic immutable memory textures via stock `gpui::RenderImage` |
| **`UNKNOWN`** | 0 | 0.0% | Complete closure; zero unclassified baseline fixtures |
| **`SHADER_REQUIRED`**| 0 | 0.0% | Forbidden in RENDER-01; composition ceiling thoroughly measured first |
| **Total** | 46 | 100.0% | Complete canonical confrontation |

---

## 4. Dependency & Protocol Audits

### 4.1 Dependency Audit
- **`image = "0.25"`**: Canonical runtime dependency in `Shelly.Ui.Gpui/Cargo.toml`. Required by `texture.rs` to construct `image::Frame` and `image::RgbaImage` passed directly into stock GPUI `RenderImage::new(vec![frame])`.
- **`walkdir`**: Audited across the codebase. Confirmed **not present** in `Cargo.toml`. Zero unused runtime dependencies.

### 4.2 Control Protocol v3 Verification
- **Protocol Version Bump**: Formally documented in `Shelly.Ui.Gpui/src/control/protocol.rs`. Bumped from v2 to v3 for `ControlCommand::RenderLabStyle` and schema v2 manifest fields (`recipe`, `seed`, `diagnostics`).
- **Cold CLI Verification**: `shelly-gpui render-lab status --json` returns valid offline static manifest (`gui_running: false`, `catalog_count: 46`).
- **Warm CLI Verification**: `shelly-gpui status`, `render-lab status`, and `render-lab ledger` (both text table and JSON format) operate with full telemetry.
- **Single-Instance Invariant**: `shelly-gpui open` focuses the existing window without duplicate process spawning.
- **Error Handling**: Invalid commands (e.g. `render-lab quality ultra`) cleanly exit with code 1 and explanatory diagnostics.
- **Appearance Subsystem**: Existing commands (`appearance status`, `appearance resolve`) remain 100% operational and unaffected.

---

## 5. ASTRA Forensic Replay Provenance & Fluid Disposition

- **Stages 1–7 Replay**: Successfully replayed all analytical SDF math, 32 material models, native GPUI gallery, and Vulkan/SPIR-V compute shaders on NVIDIA RTX 3070 (Vulkan 1.4).
- **Stage 8 Fluid Simulation**: Replayed and executed `fluid.comp` across 90 timesteps. Discovered unbounded velocity divergence due to pressure projection instability ($\nabla \cdot \mathbf{u} \neq 0$).
- **Disposition**: Formally preserved as an instructive technical failure. Fluid simulation is **not promoted** and is deferred to RENDER-04/05 under strict stability conditions.
