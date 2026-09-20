# Shelly GPUI — Slice-05 Acceptance Matrix

## 1. Traceability & Requirement Verification

This matrix maps every requirement specified in the Slice-05 plan to its concrete implementation, verification method, and automated test proof.

| Requirement ID | Requirement Description | Implementation Location | Verification Method | Automated Test Proof | Status |
|---|---|---|---|---|---|
| **REQ-05-01** | Centralize UI density & dimensions in `UiMetrics` | `src/ui_metrics.rs` | Unit testing & code inspection | `ui_metrics::tests::test_card_wrapper_vs_content_height_invariants`, `test_table_row_height_invariants` | **PASSED** |
| **REQ-05-02** | Enforce Inspector width $\ge 320\text{px}$ across 1024, 1280, 1600 viewports | `src/views/package_workstation.rs` | Dynamic mathematical splitter clamp | `views::package_workstation::tests::test_splitter_guarantees_inspector_minimum_across_reference_viewports` | **PASSED** |
| **REQ-05-03** | Compile-time SVG asset loader with 22 icons | `src/icons.rs`, `src/icons/*.svg` | AssetSource implementation via `include_bytes!` | `icons::tests::test_all_app_icons_resolve_embedded_svg`, `test_unknown_icon_returns_none` | **PASSED** |
| **REQ-05-04** | Complete elimination of emoji & lightning glyphs | `src/` | Grep verification across all source files | Verified 0 unicode emojis in `Shelly.Ui.Gpui/src/` | **PASSED** |
| **REQ-05-05** | Shared `ViewModeSwitcher` segmented control | `src/components/view_mode_switcher.rs` | Component rendering & event wiring | `state::session::tests::test_view_mode_and_inspector_tab_metadata` | **PASSED** |
| **REQ-05-06** | Complete English localization across all views | `src/views/`, `src/components/`, `src/state/` | String audit across codebase | French strings eliminated from runtime UI | **PASSED** |
| **REQ-05-07** | Restructure SettingsView into 5 logical categories | `src/views/settings.rs` | Section layout & props definition | `views::settings::tests::test_settings_clean_state_and_draft_isolation` | **PASSED** |
| **REQ-05-08** | Remove unbacked ghost settings (`shelly_search_enabled`, `no_confirm`) | `src/views/settings.rs` | UI audit & schema clean-up | Unbacked fields removed from UI rendering | **PASSED** |
| **REQ-05-09** | Provide "Reset Changes" action & dirty state badge | `src/views/settings.rs` | Draft reset method & conditional UI element | `views::settings::tests::test_settings_reset_restores_saved_state`, `test_settings_draft_dirty_flag` | **PASSED** |
| **REQ-05-10** | Single configuration snapshot at boot | `src/main.rs`, `src/views/workspace.rs` | `WorkspaceView::with_config` | Single load call in `main.rs` before window creation | **PASSED** |
| **REQ-05-11** | Hardened `GpuiUiConfig` deserialization with defaults | `src/config.rs` | `#[serde(default)]` on all fields | `config::tests::test_gpui_config_all_fields_default_on_empty_json` | **PASSED** |
| **REQ-05-12** | Window geometry sanitization ($\ge 1024 \times 680\text{px}$) | `src/config.rs` | `ConfigManager::sanitize_window_size` | `config::tests::test_sanitize_window_size_enforces_minimums` | **PASSED** |
| **REQ-05-13** | Discrete window bounds sampling at Save | `src/views/workspace.rs` | `window.window_bounds()` in `save_settings` | Verified in `WorkspaceView::save_settings` | **PASSED** |
| **REQ-05-14** | Workspace destination persistence via `last_workspace_destination` and `with_initial_tab` | `src/state/session.rs`, `src/views/workspace.rs` | Tab tracking ignoring Settings | `state::session::tests::test_last_workspace_destination_tracks_non_settings`, `test_startup_session_preserves_last_workspace_destination_across_settings` | **PASSED** |
| **REQ-05-15** | Single busy execution authority (`ConsoleModel::is_running()`) | `src/state/console.rs` | Method query & selective event subscription | `state::console::tests::test_console_is_running_authority` | **PASSED** |
| **REQ-05-16** | Active source filtering in unified search & filter bar | `src/views/workspace.rs`, `src/components/unified_search.rs` | Conditional execution & pill visibility | Verified multi-source search query dispatch | **PASSED** |
| **REQ-05-17** | Source filter reconciliation on Save | `src/views/workspace.rs` | Automatic fallback to `SourceFilter::All` | Verified in `WorkspaceView::save_settings` | **PASSED** |
| **REQ-05-18** | Real package management deletion flags with Flatpak `--remove-config` support | `src/backend/client.rs`, `src/views/workspace.rs` | `ShellyClient::build_remove_args` with `--cascade` (ALPM) & `--remove-config` (ALPM + Flatpak) | `backend::client::tests::test_build_remove_args_standard_cascade_and_remove_configs`, `test_build_remove_args_standard_no_flags`, `test_build_remove_args_flatpak_remove_configs_no_cascade`, `test_build_remove_args_flatpak_no_flags` | **PASSED** |
| **REQ-05-19** | Desktop entry English defaults & French translations | `assets/com.shellyorg.shelly-gpui.desktop` | XDG desktop file inspection | Verified keys: `GenericName`, `Comment`, `[fr]` | **PASSED** |
| **REQ-05-20** | Strict package build checking (`PKGBUILD-gpui`) | `PKGBUILD-gpui` | Locked dependencies & non-concealing phases (`fetch --locked`, `build --release --locked`, `test --release --locked`, polkit install) | Verified non-zero exit code on failure, zero error suppression | **PASSED** |
| **REQ-05-21** | Interactive news feed with URL opening | `src/views/news.rs` | `cx.open_url` on news card click and Enter/Space | Verified hover feedback, focus ring, and `AppIcon::ExternalUrl` | **PASSED** |
| **REQ-05-22** | Complete Dark and Light theme parity & contrast | `src/theme.rs` | Token mirroring & assertion test | `theme::tests::test_theme_dark_and_light_parity_and_contrast` | **PASSED** |
| **REQ-05-23** | Zero-deadcode policy enforcement | All source files | Compiler directives `#![deny(...)]` | `cargo check` and `cargo test` pass with 0 warnings | **PASSED** |
| **REQ-05-24** | Zero linter warning policy | All source files | `cargo clippy -- -D warnings` | Clippy passes with 0 warnings without suppressions | **PASSED** |
| **REQ-05-25** | Formatted code standards | All source files | `cargo fmt --check` | 0 formatting diffs across codebase | **PASSED** |
| **REQ-05-26** | Keyboard focus & Tab navigation across primary interactive controls | `src/views/workspace.rs`, `src/components/sidebar.rs`, `src/components/view_mode_switcher.rs`, `src/views/settings.rs`, `src/views/news.rs`, `src/components/inspector_header.rs` | Element state focus model, `window.focus_next/prev()`, Enter/Space activation | Verified stable element IDs, focus styling (`border_focus`), and keydown handlers | **PASSED** |
