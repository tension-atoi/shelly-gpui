# Shelly GPUI — Slice-03 Acceptance Matrix

**Target Slice**: SLICE-03 — Package Surface & Semantic Inspector  
**Baseline Git HEAD**: `216831c1450aeee95a76e159d7579b0e3340b236`  
**Implementation Git HEAD**: `abf88db66158a1744176181ddb45528350aed92a`  
**Closure Git HEAD**: `5656537877eebec5626ebc8ff61df9e38f615598`  
**Standard**: Hard Compiler Enforcement (`deny(dead_code)`, `deny(unused_variables)`, `deny(unused_imports)`, `deny(unused_must_use)`)

======================================================================
EPISTEMOLOGICAL EVIDENCE CLASSIFICATION
======================================================================

Statements and acceptance verifications are strictly categorized:
- **`RUNTIME VERIFIED`**: Directly executed and visually/behaviorally verified in live Hyprland Wayland graphical session (`xwayland: false`).
- **`TESTED`**: Verified through automated Rust unit test suite (`cargo test`, 21 tests green).
- **`CODE PATH ESTABLISHED`**: Verified through code analysis and type-level invariants with active call sites.
- **`NOT AVAILABLE ON TEST HOST`**: Honestly documented when host environment lacks entities (e.g. 0 AppImages registered on host).

======================================================================
ACCEPTANCE MATRIX
======================================================================

| Feature / Contract | Requirement | Evidence Class | Verification Details |
|---|---|---|---|
| **Dual View Mode** | View Switcher Toggle | **`RUNTIME VERIFIED`** | Pill toggle (`▦ Cards` / `☰ Table`) rendered in search filter bar. Verified in `evidence_slice03_cards.png` and `evidence_slice03_table.png`. |
| **Dual View Mode** | Fixed Table Header | **`RUNTIME VERIFIED`** | Fixed 32px header with columns `NAME`, `VERSION`, `SOURCE`, `SIZE`, `STATUS` anchored as sibling above the virtual list. Verified during vertical scrolling in `evidence_slice03_table.png` and `evidence_slice03_table_scrolled.png`. |
| **Dual View Mode** | Fixed 36px Table Rows | **`RUNTIME VERIFIED`** | High-density 36px fixed-height rows with text truncation and badges. Verified in `evidence_slice03_table.png` and `evidence_slice03_table_scrolled.png`. |
| **Dual View Mode** | Shared Results & Selection Continuity | **`TESTED`** & **`RUNTIME VERIFIED`** | Toggling Cards ↔ Table preserves identical `search_query`, `store.active_results`, and `selected_package_key` without subprocess re-fetch. Tested in `test_view_mode_continuity_preserves_selection_query_and_generation`. |
| **Dual View Mode** | Truthful Size Formatting | **`TESTED`** & **`RUNTIME VERIFIED`** | `format_bytes` outputs truthful B/KiB/MiB/GiB. Uninstalled AUR packages show `—`. Tested in `test_format_bytes` and `test_display_size_truthfulness`. |
| **Inspector Header** | Hero Identity & Badges | **`RUNTIME VERIFIED`** | Hero title, version, source badge, status pill, and repository/remote badge. Verified in `evidence_slice03_overview.png`, `evidence_slice03_flatpak.png`. |
| **Inspector Header** | Capability Action Buttons | **`RUNTIME VERIFIED`** & **`CODE PATH ESTABLISHED`** | Install/Remove/Upgrade buttons derived from package state. Code path wired to `run_package_mutation`; visual rendering verified in `evidence_slice03_overview.png`. |
| **Inspector Header** | Copy Install Command Feedback | **`RUNTIME VERIFIED`** & **`TESTED`** | Canonical install command generated and copied to clipboard; visual feedback state toggled with 2s decay. Tested in `test_canonical_install_command`. |
| **Inspector Tabs** | 3 Discrete Tabs Bar | **`RUNTIME VERIFIED`** | Tab bar with `Overview`, `Dependencies`, `Files & Build`. Active tab highlighted. Verified across all inspector screenshots. |
| **Overview Tab** | Metadata Grid & Semantic URL | **`RUNTIME VERIFIED`** & **`CODE PATH ESTABLISHED`** | Repositories, licenses, installed/download sizes, URL, maintainer, install reason. External URL click wired to `cx.open_url`. Verified in `evidence_slice03_overview.png` and `evidence_slice03_flatpak.png`. |
| **Dependencies Tab**| Typed Dependency Groups | **`RUNTIME VERIFIED`** | Runtime dependencies and Required By groups with pill badges. Verified in `evidence_slice03_dependencies.png`. |
| **Dependencies Tab**| Structured Dependency Parser| **`TESTED`** | `DependencyRef::parse` correctly parses version constraints (`>=`, `<=`, `=`) and descriptions. Tested in 3 unit tests. |
| **Dependencies Tab**| Semantic Navigation | **`TESTED`** & **`CODE PATH ESTABLISHED`** | Clicking dependency pill navigates to clean package name in Browse view without constraint/description noise. Tested in `test_dependency_navigation_preserves_clean_package_name`. |
| **Files & Build Tab**| Live AUR PKGBUILD Fetch | **`RUNTIME VERIFIED`** & **`TESTED`** | Live PKGBUILD fetched via `shelly search aur <pkg> -p`, cached in `PackageStore`, and displayed in monospaced viewer. Verified in `evidence_slice03_files_build.png` and `test_pkgbuild_cache_storage_and_invalidation`. |
| **Files & Build Tab**| ALPM Build Metadata & Honest Gap Notice | **`RUNTIME VERIFIED`** | Build date and install date displayed; capability gap for file tree listing honestly explained without synthetic mocks. Verified in `evidence_slice03_alpm_files_build.png`. |
| **Multi-Source Coverage**| Official ALPM Packages | **`RUNTIME VERIFIED`** | Search, table, cards, details, build metadata verified live with `ripgrep` and `python-absl`. |
| **Multi-Source Coverage**| AUR Packages | **`RUNTIME VERIFIED`** | Search, table, details, live PKGBUILD script verified live with `paru`. |
| **Multi-Source Coverage**| Flatpak Packages | **`RUNTIME VERIFIED`** | Search, details, flathub repo, download size (120.6 MiB), installed size (321.2 MiB), and upstream URL verified live with `org.mozilla.firefox`. Verified in `evidence_slice03_flatpak.png`. |
| **Multi-Source Coverage**| AppImage Packages | **`NOT AVAILABLE ON TEST HOST`** & **`TESTED`** | Host system returns empty AppImage catalog (`shelly list appimage -j` returns `[]`). Search filtering and key identity verified in `test_appimage_filtering_and_key_identity`. |
| **Quality & Build**| Zero Deadcode Enforcement | **`TESTED`** | `#![deny(dead_code, unused_variables, unused_imports, unused_must_use)]` strictly enforced with 0 warnings. |
| **Quality & Build**| Automated Test Suite | **`TESTED`** | 21/21 unit tests pass cleanly in `cargo test`. |
| **Quality & Build**| Release Binary Build | **`TESTED`** | `cargo build --release` produces optimized binary (`/mnt/workbench/target/release/shelly-gpui`, 20.8 MB). |
