# 08 — Phase UX-01 Acceptance Matrix

| Requirement | Specification | Implementation | Verification Method | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Strict Inspector 3/7 Cap** | $w_{\text{inspector}} \le \min(520\text{px}, w_{\text{content}} \times \frac{3}{7})$ | `UiMetrics::INSPECTOR_MAX_WIDTH = 520.0`, `UiMetrics::INSPECTOR_MAX_RATIO = 3.0 / 7.0`, clamped in `compute_splitter_bounds` | `views::package_workstation::tests::test_inspector_ratio_pure_math_and_bounds`, `test_splitter_guarantees_inspector_bounds_across_reference_viewports` | **PASSED** |
| **Inspector Usable Minimum** | $w_{\text{inspector}} \ge 320\text{px}$ in horizontal mode | `UiMetrics::INSPECTOR_MIN_USABLE = 320.0`, clamped as upper bound for $L_{\max}$ | Unit test `test_splitter_guarantees_inspector_bounds_across_reference_viewports` across [750, 900, 1024, 1280, 1600, 1920] | **PASSED** |
| **Adaptive Layout Mode** | `Horizontal` when $w_{\text{content}} \ge 746.67\text{px}$, `Stacked` below | `WorkstationLayoutMode::from_content_width` using `HORIZONTAL_SPLIT_MIN_CONTENT_WIDTH = 320 * (7/3)` | Unit test `views::package_workstation::tests::test_layout_mode_adaptation_at_boundary` | **PASSED** |
| **Zero Overlay Invariant** | Inspector never floats, occludes, or covers list results | Inspector docked side-by-side in `Horizontal` mode; docked below list in `Stacked` mode | Code audit of `package_workstation.rs` lines 980-1021; Wayland screenshot | **PASSED** |
| **Sticky Query Surface** | Search bar and filters remain sticky at top; results scroll beneath | `top_bar` placed above `uniform_list` container in `list_pane` | Code audit of `list_pane` flex column hierarchy; runtime verification | **PASSED** |
| **Zero Deadcode Policy** | No dead code, no unused variables/imports, no warning suppressions | Crates enforced with `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, `#![deny(unused_must_use)]` | `cargo clippy --release --locked -- -D warnings` exits 0 with zero warnings | **PASSED** |
| **Release Locked Tests** | All unit and integration tests passing in release locked profile | 93 passing tests in release locked profile | `cargo test --release --locked` exits 0 (93 passed) | **PASSED** |
| **Native Packaging** | AUR PKGBUILD builds clean package from remote git commit | Built via `makepkg -C -c -f`, installed via `pacman -U` | `pacman -Q shelly-gpui-git` (`r4687.g3fd657b3-1`) and byte integrity check | **PASSED** |
| **Wayland Runtime Execution** | App launches under native Wayland Hyprland compositor with active window | `/proc/<pid>/exe -> /usr/lib/shelly/shelly-gpui-bin`, Hyprland `xwayland: 0` | PID 2733610, `hyprctl clients -j`, Wayland screenshot via `grim` | **PASSED** |

