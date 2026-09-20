# 08 — Acceptance Matrix (Phases UX-01 & UX-02)

## Phase UX-01 Acceptance Matrix

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

---

## Phase UX-02 Acceptance Matrix (Query Workbench Redesign)

| Requirement | Specification | Implementation | Verification Method | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Eradication of Floating Pills** | Zero floating pill chips in Query Workbench; `render_quick_pill` deleted | Codebase completely clean of quick pill rendering | Code audit of `query_workbench.rs`, `grep` check, screenshot review | **PASSED** |
| **Dominant Native Search Input** | Full-width input with tactile states (focus, hover, clear `×`, search glyph) | `SearchInputView` with `w_full()`, border ring, clear action | Wayland captures `evidence_ux02_01`, `evidence_ux02_06`; unit tests in `search_input.rs` | **PASSED** |
| **Desktop Native Toolbar (Row 2)** | Uniform 28px native toolbar with disclosure button, state dropdown, sort dropdown, and view switcher | `render_toolbar` flex row with `render_filter_button`, `render_state_button`, `render_sort_button`, and `ViewModeSwitcher` | Wayland captures `evidence_ux02_01`, `evidence_ux02_06`, `evidence_ux02_08` | **PASSED** |
| **Filters Disclosure Popover** | Multi-source checkboxes (Official, AUR, Flatpak, AppImage) + state radios + section dividers | `MenuSurface` with `MenuCheckItem` and `MenuRadioItem` | Wayland capture `evidence_ux02_02_filters_menu_open.png`; unit tests | **PASSED** |
| **Active Filter Badge Counter** | Button shows badge with count of active non-default filters: `Filters (N) ▾` | `compute_filter_badge_count` pure math function | Wayland captures `evidence_ux02_03`, `evidence_ux02_04b`; unit tests | **PASSED** |
| **Active Filter Textual Summary** | Single clean row replacing chip pile: `Showing X packages • Sources: ... • State: ...` | `format_active_filter_summary` pure formatting function | Wayland captures `evidence_ux02_03`, `evidence_ux02_04b`; unit tests | **PASSED** |
| **Inline Clear Filters Action** | Tactile `× Clear filters` button resetting scope and state with zero layout jank | `on_clear_filters` wired to session reset; Row 3 collapses cleanly | Wayland capture `evidence_ux02_04c_filters_cleared.png`; session unit test | **PASSED** |
| **Zero Deadcode & Warnings** | Enforce `#![deny(dead_code)]` and zero clippy warnings | All components and handlers wired; strict YAGNI followed | `cargo clippy --release --locked -- -D warnings` exits 0 | **PASSED** |
| **Release Locked Tests** | 99/99 tests passing in release locked profile | Added unit test suite for Query Workbench filters, summary, and session | `cargo test --release --locked` exits 0 (99 passed) | **PASSED** |
| **Wayland Runtime Deployment** | Binary packaged and executed live under Hyprland Wayland compositor | Packaged as `shelly-gpui-git r4689.g73143130-1`, active window verified | PID 2957817, client `0x5651973c84a0`, 18 runtime screenshots captured | **PASSED** |

---

## Phase UX-02P Acceptance Matrix (Command Surface Productization)

| Requirement | Specification | Implementation | Verification Method | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Unboxed Row 2 Toolbar** | No permanent perimeter borders/boxes at rest; calm surface; hover reveals fill | Transparent borders/backgrounds at rest; subtle hover fill (`bg_surface_hover`); active states with accent | Wayland runtime verification; dark & light theme audit | **PASSED** |
| **Permanent Status Rail (Row 3, 24px)** | Fixed height 24px container for package count and status; zero layout shift | `format_status_rail`, `StatusRailState`, fixed `h(px(24.0))` container | Unit tests `test_format_status_rail_*`; zero layout shift verified | **PASSED** |
| **Non-Redundant Filtering** | `Filters` contains exclusively sources in Wide/Medium; appends State dynamically only in Narrow | Dynamic inclusion conditioned on `WorkbenchBreakpoint::Narrow` | Unit tests in `query_workbench.rs` and `package_workstation.rs` | **PASSED** |
| **Desktop Checkmark Menus** | Single-select menus use native desktop checkmark `✓` (no radio circles) | `MenuCheckmarkItem` with active `✓` glyph and alignment spacers | Code audit of `menu.rs` and `query_workbench.rs`; runtime screenshots | **PASSED** |
| **Kinematic Menu Lifecycle** | Opening (150ms ease-out quint), Open, Closing (100ms ease-out quint), Closed | `MenuLifecycle` with opacity and translateY interpolation | Unit test `test_menu_lifecycle_transitions`; runtime verification | **PASSED** |
| **Keyboard Contract & Roving Focus** | `↑ ↓ Home End Enter Space Escape` navigation; focus return to trigger button upon close | `navigate_menu`, `FocusHandle` tracking for trigger buttons | Manual keyboard navigation audit; GPUI focus tests | **PASSED** |
| **Local Search Focus Interpolation** | 110ms smooth transition for focus/hover with subtle lift shadow & icon tint | `search_focus_anim` animation with local state tracking | Code audit in `search_input.rs`; runtime verification | **PASSED** |
| **Compact Search Indicator** | Compact 14px indicator (7px pulsing accent dot) replacing verbose text; `"/"` shortcut hint | Pulsing dot in `search_input.rs` and `"/"` hint when empty | Code audit; runtime verification | **PASSED** |
| **Accessible Clear Button (`×`)** | `focusable().tab_stop(true)` with Enter/Space keyboard listeners | Full keyboard event listener on `search_clear_button` | Code audit; unit test in `search_input.rs` | **PASSED** |
| **Responsive Breakpoints** | Wide (>= 820px), Medium (600-819px), Narrow (< 600px) | `WorkbenchBreakpoint::from_width` pure classification | Unit test `test_query_workbench_breakpoints_wide_medium_narrow` | **PASSED** |
| **Exact-SHA Build & Package** | Built and packaged via Arch PKGBUILD matching exact git commit SHA | `makepkg -C -c -f`, `pacman -U`, `pacman -Q` verification | Verified with pacman package query | **PASSED** |
| **Zero Deadcode & 100/100 Tests** | `#![deny(dead_code)]` with zero warnings; 100% tests passing | Strict YAGNI, 100 unit tests passing in release locked profile | `cargo test --release --locked` exits 0 (100 passed) | **PASSED** |
