# Shelly GPUI — Slice-03 Acceptance Matrix

**Target Slice**: SLICE-03 — Package Surface & Semantic Inspector  
**Baseline**: `main`  
**Standard**: Hard Compiler Enforcement (`deny(dead_code)`, `deny(unused_variables)`, `deny(unused_imports)`, `deny(unused_must_use)`)

======================================================================
EPISTEMOLOGICAL EVIDENCE CLASSIFICATION
======================================================================

Statements and acceptance verifications are strictly categorized:
- **`RUNTIME VERIFIED`**: Directly executed and visually/behaviorally verified in live Hyprland Wayland graphical session (`xwayland: false`).
- **`TESTED`**: Verified through automated Rust unit test suite (`cargo test`).
- **`ESTABLISHED`**: Directly verified by code review and type-level invariants.
- **`CODE PATH ESTABLISHED`**: Execution pathway and command invocation logic are fully wired in code.

======================================================================
ACCEPTANCE MATRIX
======================================================================

| Feature / Contract | Requirement | Evidence Class | Verification Details |
|---|---|---|---|
| **Dual View Mode** | View Switcher Toggle | **`RUNTIME VERIFIED`** | Pill toggle (`▦ Cards` / `☰ Table`) rendered in search filter bar. Verified in `evidence_slice03_cards.png` and `evidence_slice03_table.png`. |
| **Dual View Mode** | Table Sticky Header | **`RUNTIME VERIFIED`** | Sticky 32px header with columns `NAME`, `VERSION`, `SOURCE`, `SIZE`, `STATUS`. Verified in `evidence_slice03_table.png`. |
| **Dual View Mode** | Fixed 36px Table Rows | **`RUNTIME VERIFIED`** | High-density 36px fixed-height rows with text truncation and badges. Verified in `evidence_slice03_table.png`. |
| **Dual View Mode** | Shared Results & Key Invariant | **`TESTED`** & **`RUNTIME VERIFIED`** | Changing `view_mode` preserves active search results and selection without re-fetching. Verified in `test_view_mode_and_inspector_tab_metadata`. |
| **Dual View Mode** | Truthful Size Formatting | **`TESTED`** & **`RUNTIME VERIFIED`** | `format_bytes` outputs B/KiB/MiB/GiB. Uninstalled AUR packages show `—`. Tested in `test_format_bytes` and `test_display_size_truthfulness`. |
| **Inspector Header** | Hero Identity & Badges | **`RUNTIME VERIFIED`** | Hero title, version, source badge, status pill, and repository badge. Verified in `evidence_slice03_overview.png`. |
| **Inspector Header** | Capability Action Buttons | **`RUNTIME VERIFIED`** | Install/Remove buttons derived from package state. Verified in `evidence_slice03_overview.png`. |
| **Inspector Header** | Copy Command Feedback | **`RUNTIME VERIFIED`** & **`CODE PATH ESTABLISHED`** | Canonical install command copied to clipboard; visual feedback state toggled. Verified in `evidence_slice03_overview.png`. |
| **Inspector Tabs** | 3 Discrete Tabs Bar | **`RUNTIME VERIFIED`** | Tab bar with `Overview`, `Dependencies`, `Files & Build`. Active tab highlighted. Verified in all 3 inspector screenshots. |
| **Overview Tab** | Metadata Grid | **`RUNTIME VERIFIED`** | Repositories, licenses, installed/download sizes, URL, packager, install reason. Verified in `evidence_slice03_overview.png`. |
| **Dependencies Tab**| Typed Dependency Groups | **`RUNTIME VERIFIED`** | Runtime dependencies and Required By groups with pill badges. Verified in `evidence_slice03_dependencies.png`. |
| **Dependencies Tab**| Structured Dependency Parser| **`TESTED`** | `DependencyRef::parse` correctly parses constraints (`>=`, `<=`, `=`) and descriptions. Tested in 3 unit tests. |
| **Files & Build Tab**| Live AUR PKGBUILD Fetch | **`RUNTIME VERIFIED`** & **`TESTED`** | Live PKGBUILD fetched via `shelly search aur <pkg> -p`, cached in `PackageStore`, and displayed in monospaced code viewer. Verified in `evidence_slice03_files_build.png` and `test_pkgbuild_cache_storage_and_invalidation`. |
| **Files & Build Tab**| Honest Capability Notices | **`RUNTIME VERIFIED`** | Explains ALPM archive content constraint without crashing or faking data. |
| **Semantic Model** | Canonical Install Commands | **`TESTED`** | Generates valid Shelly CLI commands for ALPM, AUR, Flatpak. Tested in `test_canonical_install_command`. |
| **Quality & Build**| Zero Deadcode Enforcement | **`TESTED`** | `#![deny(dead_code, unused_variables, unused_imports, unused_must_use)]` strictly enforced with 0 warnings. |
| **Quality & Build**| Automated Test Suite | **`TESTED`** | 19/19 unit tests pass cleanly in `cargo test`. |
| **Quality & Build**| Release Binary Build | **`TESTED`** | `cargo build --release` produces optimized binary (`shelly-gpui`). |
