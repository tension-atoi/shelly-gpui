# Shelly GPUI — Slice-03 Acceptance Report

**Date**: 2026-09-20  
**Target Slice**: SLICE-03 — Package Surface & Semantic Inspector  
**Baseline HEAD**: `main`  
**Compositor**: Hyprland 0.56.2 (`WAYLAND_DISPLAY=wayland-1`, native Wayland client `xwayland: false`)  
**Zero Deadcode Policy**: Hard Compiler Enforcement (`deny(dead_code)`, `deny(unused_variables)`, `deny(unused_imports)`, `deny(unused_must_use)`)  

---

## 1. Executive Summary

Slice-03 achieves full closure of the "Package Surface & Semantic Inspector" specification. The dual package viewing surface (Cards vs Table) is fully realized with shared results and selection stability. The package inspector has been completely decomposed from an ad-hoc view into a coordinated, capability-driven 3-tab architecture (`Overview`, `Dependencies`, `Files & Build`). Live AUR PKGBUILD recipes are fetched directly from upstream Shelly and rendered with syntax clarity.

All code adheres strictly to the Zero Deadcode Policy, with 19/19 unit tests passing and 5 empirical runtime screenshots captured under a live native Wayland compositor.

---

## 2. Telemetry & Verified Test Results

### Automated Unit Test Suite (`cargo test`)
```text
running 19 tests
test components::package_table::tests::test_format_bytes ... ok
test state::console::tests::test_clear_logs_preserves_lifecycle_status ... ok
test components::package_table::tests::test_display_size_truthfulness ... ok
test state::console::tests::test_console_model_defaults ... ok
test state::package_store::tests::test_appimage_filtering_and_key_identity ... ok
test state::package_store::tests::test_pkgbuild_cache_storage_and_invalidation ... ok
test state::package_store::tests::test_package_store_cache_and_invalidation ... ok
test state::package_store::tests::test_search_key_normalization ... ok
test state::semantic::tests::test_canonical_install_command ... ok
test state::package_store::tests::test_search_cache_storage ... ok
test state::semantic::tests::test_dependency_ref_parse_unversioned ... ok
test state::semantic::tests::test_dependency_ref_parse_versioned ... ok
test state::semantic::tests::test_package_capabilities_derive ... ok
test state::semantic::tests::test_dependency_ref_parse_with_description ... ok
test state::session::tests::test_app_session_defaults ... ok
test state::session::tests::test_nav_destination_metadata ... ok
test state::session::tests::test_source_filter_metadata ... ok
test state::session::tests::test_package_key_equality_and_hashing ... ok
test state::session::tests::test_view_mode_and_inspector_tab_metadata ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Compiler Quality & Linter Gates
- `cargo fmt --check`: 100% compliant (0 diffs).
- `cargo clippy`: 0 errors, 0 warnings under `#![deny(dead_code, unused_variables, unused_imports, unused_must_use)]`.
- `cargo build --release`: Clean release compilation producing `/mnt/workbench/target/release/shelly-gpui` (20.8 MB).

---

## 3. Native Wayland Runtime Evidence

The release binary was executed on Wayland display `wayland-1` under Hyprland 0.56.2 without XWayland translation. All five required evidence views were captured directly from the compositor framebuffer via `grim`:

1. [`evidence_slice03_cards.png`](evidence_slice03_cards.png):
   - **Captured View**: Initial Browse landing in default Cards view mode.
   - **Key Elements**: Workstation sidebar, empty discovery state hero, search bar input, source filter pills, view switcher toggle (`▦ Cards` active).
   - **Evidence Class**: `RUNTIME VERIFIED`

2. [`evidence_slice03_table.png`](evidence_slice03_table.png):
   - **Captured View**: High-density Table view with search query `ripgrep`.
   - **Key Elements**: Active `☰ Table` view toggle, 32px sticky header (`NAME`, `VERSION`, `SOURCE`, `SIZE`, `STATUS`), 36px virtualized table rows with source badges (`ALPM`, `AUR`), truthful size formatting (`3.6 MiB`, `12.5 MiB`, `—`), and `Available` status pills.
   - **Evidence Class**: `RUNTIME VERIFIED`

3. [`evidence_slice03_overview.png`](evidence_slice03_overview.png):
   - **Captured View**: Selected package `ripgrep` with active Overview tab.
   - **Key Elements**: Hero header with title, version, badges, capability-driven `Install` button, `Copy install command` button, tab bar with active `📋 Overview` tab, description, and 6-cell metadata grid with interactive upstream GitHub link.
   - **Evidence Class**: `RUNTIME VERIFIED`

4. [`evidence_slice03_dependencies.png`](evidence_slice03_dependencies.png):
   - **Captured View**: Selected package `ripgrep` with active Dependencies tab.
   - **Key Elements**: Active `🔗 Dependencies` tab, `RUNTIME DEPENDENCIES (3)` group with interactive pills (`glibc`, `libgcc`, `pcre2`), and `REQUIRED BY (9)` reverse dependency pills (`bat-extras`, `code`, `cursor-bin`, etc.).
   - **Evidence Class**: `RUNTIME VERIFIED`

5. [`evidence_slice03_files_build.png`](evidence_slice03_files_build.png):
   - **Captured View**: Selected AUR package `paru` with active Files & Build tab.
   - **Key Elements**: Active `🛠️ Files & Build` tab, live PKGBUILD script fetched from AUR via `shelly search aur paru -p`, rendered inside a styled monospaced viewer with full recipe definitions (`pkgname`, `pkgver`, `prepare()`, `build()`, `package()`).
   - **Evidence Class**: `RUNTIME VERIFIED`

---

## 4. Acceptance Signoff

| Contract Item | Status | Notes |
|---|---|---|
| Dual Virtualized Package Surface | **ACCEPTED** | Cards and Table modes sharing identical data structures and selection keys |
| Decomposed 3-Tab Inspector | **ACCEPTED** | Overview, Dependencies, and Files & Build tabs fully coordinated |
| Typed Semantic Domain Models | **ACCEPTED** | Parsed `DependencyRef`, `PackageCapabilities`, `SemanticTarget`, and canonical install commands |
| Truthful Size & Capability Telemetry | **ACCEPTED** | No phantom numbers or misleading file tree mocks |
| Zero Deadcode Policy | **ACCEPTED** | Hard compiler enforcement, 0 unused items, 0 warning suppressions |
| Full Test & Runtime Verification | **ACCEPTED** | 19 unit tests passing, 5 native Wayland screenshots verified |
