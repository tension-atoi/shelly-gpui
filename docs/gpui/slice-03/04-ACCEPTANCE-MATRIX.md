# Shelly GPUI — Slice-03 Acceptance Matrix

**Target Slice**: SLICE-03 — Package Surface & Semantic Inspector  
**Baseline Git HEAD**: `216831c1450aeee95a76e159d7579b0e3340b236`  
**Implementation Git HEAD**: `abf88db66158a1744176181ddb45528350aed92a`  
**Behavioral Evidence Closure Git HEAD**: `e4737cc66d0afd5aa43b12818838f90e878d8d8a`  
**Final Verification & Scaffolding Cleanup**: Current commit on `main`  
**Standard**: Hard Compiler Enforcement (`deny(dead_code)`, `deny(unused_variables)`, `deny(unused_imports)`, `deny(unused_must_use)`)

======================================================================
EPISTEMOLOGICAL EVIDENCE CLASSIFICATION
======================================================================

Statements and acceptance verifications are strictly categorized:
- **`RUNTIME VERIFIED`**: Directly executed and visually/behaviorally verified in live Hyprland Wayland graphical session (`xwayland: false`).
- **`TESTED`**: Verified through automated Rust unit test suite (`cargo test`, 21 tests green).
- **`ESTABLISHED`**: Directly verified by code review and architectural type-level invariants with real callers.
- **`CODE PATH ESTABLISHED`**: Execution pathway and invocation logic are fully wired to real callers, without asserting unobserved runtime side effects.
- **`NOT RUNTIME VERIFIED`**: Explicitly noted when an external side-effect (e.g. external browser spawn, OS clipboard pasteback) was not empirically captured.
- **`NOT AVAILABLE ON TEST HOST`**: Honestly documented when host environment lacks entities (e.g. 0 AppImages registered on host).

======================================================================
ACCEPTANCE MATRIX
======================================================================

| Feature / Contract | Requirement | Evidence Class | Verification Details |
|---|---|---|---|
| **Dual View Mode** | View Switcher Toggle | **`RUNTIME VERIFIED`** | Pill toggle (`▦ Cards` / `☰ Table`) rendered in search filter bar. Verified in `evidence_slice03_cards.png` and `evidence_slice03_table.png`. |
| **Dual View Mode** | Fixed Table Header | **`ESTABLISHED`** & **`RUNTIME VERIFIED`** | Fixed 32px header with columns `NAME`, `VERSION`, `SOURCE`, `SIZE`, `STATUS` anchored as sibling above the virtual list container. Verified during vertical scrolling in `evidence_slice03_table.png` and `evidence_slice03_table_scrolled.png`. |
| **Dual View Mode** | Fixed 36px Table Rows | **`RUNTIME VERIFIED`** | High-density 36px fixed-height rows with text truncation and badges. Verified in `evidence_slice03_table.png` and `evidence_slice03_table_scrolled.png`. |
| **Dual View Mode** | AppSession Continuity | **`TESTED`** | Toggling `view_mode` preserves `search_query`, `selected_package_key`, and `search_generation`. Verified in `test_view_mode_continuity_preserves_selection_query_and_generation`. |
| **Dual View Mode** | Shared Results & No-Subprocess Invariant | **`ESTABLISHED`** | Cards and Table read the identical `store.active_results` slice; toggling `set_view_mode` does not trigger `execute_search` or spawn any CLI subprocess. |
| **Dual View Mode** | Truthful Size Formatting | **`TESTED`** & **`RUNTIME VERIFIED`** | `format_bytes` outputs truthful B/KiB/MiB/GiB. Uninstalled AUR packages show `—`. Tested in `test_format_bytes` and `test_display_size_truthfulness`. |
| **Inspector Header** | Hero Identity & Badges | **`RUNTIME VERIFIED`** | Hero title, version, source badge, status pill, and repository/remote badge. Verified in `evidence_slice03_overview.png`, `evidence_slice03_flatpak.png`. |
| **Inspector Header** | Capability Action Buttons | **`RUNTIME VERIFIED`** & **`CODE PATH ESTABLISHED`** | Install/Remove/Upgrade buttons derived from package state (`PackageCapabilities`). Mutation streaming pipeline wired to `shelly` CLI via Tokio mpsc channels; destructive execution was deliberately not executed on host. |
| **Inspector Header** | Canonical Install Command Generation | **`TESTED`** | Generates authentic Shelly syntax: `shelly install standard <name>` (ALPM), `shelly install aur <name>` (AUR), `shelly install flatpak <name>` (Flatpak), and `None` (AppImage). Tested in `test_canonical_install_command`. |
| **Inspector Header** | Copy Command Call Path | **`CODE PATH ESTABLISHED`** | `copy_install_command` dispatches `cx.write_to_clipboard(ClipboardItem::new_string(cmd))` and sets 2s decay timer. |
| **Inspector Header** | Copied Feedback Rendering | **`RUNTIME VERIFIED`** | Transient "Copied!" button feedback state rendered on click. |
| **Inspector Header** | Clipboard Payload Readback | **`NOT RUNTIME VERIFIED`** | Clipboard contents were not read back/pasted from Wayland compositor clipboard during test run; payload correctness rests on unit tests. |
| **Inspector Tabs** | 3 Discrete Tabs Bar | **`RUNTIME VERIFIED`** | Tab bar with `Overview`, `Dependencies`, `Files & Build`. Active tab highlighted. Verified across all inspector screenshots. |
| **Overview Tab** | Metadata Grid Rendering | **`RUNTIME VERIFIED`** | Repositories, licenses, installed/download sizes, URL, maintainer, install reason. Verified in `evidence_slice03_overview.png` and `evidence_slice03_flatpak.png`. |
| **Overview Tab** | External URL Call Path | **`CODE PATH ESTABLISHED`** | External link click handler calls `cx.open_url(url.as_str())`. |
| **Overview Tab** | External App Launch | **`NOT RUNTIME VERIFIED`** | Browser/external handler launch was not verified at compositor level. |
| **Dependencies Tab**| Typed Dependency Groups | **`RUNTIME VERIFIED`** | Runtime dependencies and Required By groups with pill badges. Verified in `evidence_slice03_dependencies.png`. |
| **Dependencies Tab**| Structured Dependency Parser| **`TESTED`** | `DependencyRef::parse` correctly parses version constraints (`>=`, `<=`, `=`) and descriptions. Tested in 3 unit tests. |
| **Dependencies Tab**| Clean-Name Transition | **`TESTED`** | Navigation uses clean package name (stripping constraints and descriptions). Tested in `test_dependency_navigation_preserves_clean_package_name`. |
| **Dependencies Tab**| Click Handler Wiring | **`CODE PATH ESTABLISHED`** | Pill click event is wired to set `NavDestination::Browse`, update search query, and trigger search. |
| **Files & Build Tab**| Live AUR PKGBUILD Fetch | **`RUNTIME VERIFIED`** & **`TESTED`** | Live PKGBUILD fetched via `shelly search aur <pkg> -p`, cached in `PackageStore`, and displayed in monospaced viewer. Verified in `evidence_slice03_files_build.png` and `test_pkgbuild_cache_storage_and_invalidation`. |
| **Files & Build Tab**| ALPM Build Metadata & Gap Notice | **`RUNTIME VERIFIED`** | Build date and install date displayed; capability gap for file tree listing honestly explained without synthetic mocks. Verified in `evidence_slice03_alpm_files_build.png`. |
| **Multi-Source Coverage**| Official ALPM Packages | **`RUNTIME VERIFIED`** | Search, table, cards, details, build metadata verified live with `ripgrep` and `python-absl`. |
| **Multi-Source Coverage**| AUR Packages | **`RUNTIME VERIFIED`** | Search, table, details, live PKGBUILD script verified live with `paru`. |
| **Multi-Source Coverage**| Flatpak Packages | **`RUNTIME VERIFIED`** | Search, details, flathub repo, download size (120.6 MiB), installed size (321.2 MiB), and upstream URL verified live with `org.mozilla.firefox`. Verified in `evidence_slice03_flatpak.png`. |
| **Multi-Source Coverage**| AppImage Packages | **`NOT AVAILABLE ON TEST HOST`** & **`TESTED`** | Host system returns empty AppImage catalog (`shelly list appimage -j` returns `[]`). Search filtering and key identity verified in `test_appimage_filtering_and_key_identity`. |
| **Production Hygiene**| Zero Test Scaffolding | **`ESTABLISHED`** & **`TESTED`** | All evidence-only runtime hooks (`SHELLY_TEST_*`, `SHELLY_SELECT_PACKAGE`, `SHELLY_SOURCE_FILTER`) completely removed from `workspace.rs`. |
| **Quality & Build**| Zero Deadcode Enforcement | **`TESTED`** | `#![deny(dead_code, unused_variables, unused_imports, unused_must_use)]` strictly enforced with 0 warnings. |
| **Quality & Build**| Automated Test Suite | **`TESTED`** | 21/21 unit tests pass cleanly in `cargo test`. |
| **Quality & Build**| Release Binary Build | **`TESTED`** | `cargo build --release` produces optimized binary (`/mnt/workbench/target/release/shelly-gpui`, 20.8 MB). |
