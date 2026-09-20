# Shelly GPUI — Slice-06R Acceptance Report

## Execution Summary

- **Slice**: `SLICE-06R — Query Workbench UX, Real Menus & Failure Truth Closure`
- **Date**: September 20, 2026
- **Baseline SHA**: `88f903e843e90ec7632ba5cf01309322ba81516e`
- **Final Closure SHA**: `9cb39a8f5f242fa52e5b7b9195bdf6bb2d352cbb`
- **Compiler Standards**: `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, `#![deny(unused_must_use)]`
- **Warning Suppressions**: ZERO (`#[allow(...)]` strictly forbidden)
- **Unit Test Coverage**:
  - `Shelly.Ui.Gpui`: 90 tests passing (100% pass rate in release locked profile)
  - `Shelly.Cli.Zig`: all tests passing

---

## Verification Evidence

### 1. Hard Compiler Enforcement & Linter Cleanliness
```bash
cargo fmt --check
# Result: clean, 0 formatting errors

cargo clippy --release --locked -- -D warnings
# Result: clean, 0 warnings, 0 errors
```

### 2. Full Test Suite Execution (90/90 Passing)
```text
running 90 tests
test backend::client::tests::test_build_remove_args_flatpak_no_flags ... ok
test backend::client::tests::test_build_remove_args_flatpak_remove_configs_no_cascade ... ok
test backend::client::tests::test_build_remove_args_standard_cascade_and_remove_configs ... ok
test backend::client::tests::test_build_remove_args_standard_no_flags ... ok
test backend::protocol::tests::test_decode_malformed_base64_passthrough ... ok
test backend::protocol::tests::test_decode_ordinary_stdout_passthrough ... ok
test backend::protocol::tests::test_decode_progress_frame ... ok
test backend::protocol::tests::test_decode_valid_alpm_error_frame ... ok
test backend::protocol::tests::test_decode_valid_alpm_info_frame ... ok
test components::package_table::tests::test_display_size_truthfulness ... ok
test components::package_table::tests::test_format_bytes ... ok
test components::search_input::tests::test_search_input_backspace_and_delete ... ok
test components::search_input::tests::test_search_input_character_boundaries_utf8 ... ok
test components::search_input::tests::test_search_input_insert_and_caret_advance ... ok
test components::search_input::tests::test_search_input_select_all_and_clear ... ok
test components::search_input::tests::test_search_input_selection_replacement ... ok
test config::tests::test_gpui_config_all_fields_default_on_empty_json ... ok
test config::tests::test_gpui_config_backward_compatibility_without_reduce_motion ... ok
test config::tests::test_gpui_config_with_explicit_reduce_motion ... ok
test config::tests::test_sanitize_window_size_enforces_minimums ... ok
test icons::tests::test_all_app_icons_resolve_embedded_svg ... ok
test icons::tests::test_unknown_icon_returns_none ... ok
test state::console::tests::test_clear_logs_preserves_lifecycle_status ... ok
test state::console::tests::test_console_is_running_authority ... ok
test state::console::tests::test_console_model_defaults ... ok
test state::console::tests::test_finish_operation_failure_auto_discloses_logical_open ... ok
test state::console::tests::test_start_operation_honors_auto_open_setting ... ok
test state::motion::tests::test_animated_scalar_clamp ... ok
test state::console::tests::test_timeline_phases_initialized_and_updated ... ok
test state::motion::tests::test_animated_scalar_completion ... ok
test state::motion::tests::test_animated_scalar_forward_progress ... ok
test state::motion::tests::test_animated_scalar_inactive_no_request ... ok
test state::motion::tests::test_animated_scalar_reduced_motion_snap ... ok
test state::motion::tests::test_motion_policy_and_durations ... ok
test state::motion::tests::test_animated_scalar_retarget_reversal_from_sampled_value ... ok
test state::package_store::tests::test_appimage_filtering_and_key_identity ... ok
test state::package_store::tests::test_failure_truth_partial_and_failed_not_cached ... ok
test state::package_store::tests::test_package_store_cache_and_invalidation ... ok
test state::package_store::tests::test_pkgbuild_cache_storage_and_invalidation ... ok
test state::package_store::tests::test_search_key_normalization ... ok
test state::package_store::tests::test_search_generation_discard_stale_and_accept_fresh ... ok
test state::package_store::tests::test_search_cache_storage ... ok
test state::package_store::tests::test_source_health_and_outcome_transitions ... ok
test state::query::tests::test_multi_source_scope_selection_and_disabled_exclusion ... ok
test state::query::tests::test_package_state_filter_installed_updates ... ok
test state::query::tests::test_query_workbench_breakpoints_wide_medium_narrow ... ok
test state::query::tests::test_sort_mode_relevance_name_source_installed ... ok
test state::query::tests::test_sort_mode_stable_tie_breaking ... ok
test state::semantic::tests::test_canonical_install_command ... ok
test state::semantic::tests::test_dependency_ref_parse_unversioned ... ok
test state::semantic::tests::test_dependency_ref_parse_versioned ... ok
test state::semantic::tests::test_dependency_ref_parse_with_description ... ok
test state::semantic::tests::test_package_capabilities_derive ... ok
test state::session::tests::test_app_session_defaults ... ok
test state::session::tests::test_clear_filters_resets_scope_and_state_but_preserves_query_and_sort ... ok
test state::session::tests::test_dependency_navigation_preserves_clean_package_name ... ok
test state::session::tests::test_last_workspace_destination_tracks_non_settings ... ok
test state::session::tests::test_nav_destination_metadata ... ok
test state::session::tests::test_package_key_equality_and_hashing ... ok
test state::session::tests::test_session_epochs_increment ... ok
test state::session::tests::test_startup_session_preserves_last_workspace_destination_across_settings ... ok
test state::session::tests::test_view_mode_and_inspector_tab_metadata ... ok
test state::session::tests::test_view_mode_continuity_preserves_selection_query_and_generation ... ok
test state::session::tests::test_workspace_config_index_round_trip ... ok
test state::toast::tests::test_toast_deterministic_eviction_all_errors_evicts_oldest ... ok
test state::toast::tests::test_toast_deterministic_eviction_preserves_error ... ok
test state::toast::tests::test_toast_dismiss_transitions ... ok
test state::toast::tests::test_toast_dismiss_during_entering_prevents_visible_regression ... ok
test state::toast::tests::test_toast_lifecycle_transitions ... ok
test state::toast::tests::test_toast_max_visible_bound ... ok
test state::toast::tests::test_toast_monotonic_ids ... ok
test state::toast::tests::test_toast_reduced_motion_immediate ... ok
test state::toast::tests::test_toast_timer_target_identity ... ok
test theme::tests::test_theme_dark_and_light_parity_and_contrast ... ok
test ui_metrics::tests::test_card_wrapper_vs_content_height_invariants ... ok
test ui_metrics::tests::test_splitter_and_sidebar_geometry_invariants ... ok
test ui_metrics::tests::test_table_row_height_invariants ... ok
test views::package_workstation::tests::test_dual_usable_splitter_clamp_invariants ... ok
test views::package_workstation::tests::test_splitter_clamp_maximum ... ok
test views::package_workstation::tests::test_splitter_clamp_minimum ... ok
test views::package_workstation::tests::test_splitter_delta_forward ... ok
test views::package_workstation::tests::test_splitter_delta_reverse ... ok
test views::package_workstation::tests::test_splitter_guarantees_inspector_minimum_across_reference_viewports ... ok
test views::package_workstation::tests::test_splitter_sidebar_offset_independence ... ok
test views::settings::tests::test_reduce_motion_toggle_updates_draft ... ok
test views::settings::tests::test_settings_clean_state_and_draft_isolation ... ok
test views::settings::tests::test_settings_draft_dirty_flag ... ok
test views::settings::tests::test_settings_reset_restores_saved_state ... ok
test views::settings::tests::test_settings_save_clears_dirty_on_success ... ok
test views::settings::tests::test_settings_save_preserves_dirty_on_failure ... ok

test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 3. System Packaging & Arch Linux Installation
- **Built Package**: `shelly-gpui-git r4685.g9cb39a8f-1` (via `makepkg -C -c -f`)
- **Installation Verification**:
  ```bash
  pacman -Q shelly-gpui-git
  # shelly-gpui-git r4685.g9cb39a8f-1
  ```
- **Installed Binary Target**: `/usr/lib/shelly/shelly-gpui-bin`

### 4. Native Wayland Runtime Execution
- **Active Wayland Compositor**: Hyprland
- **Process Status**:
  - PID: Active runtime
  - `/proc/<pid>/exe` link: `/usr/lib/shelly/shelly-gpui-bin`
- **Hyprland Client Properties**:
  - `class`: `shelly-gpui`
  - `xwayland`: `0` (Native Wayland)
- **Visual Evidence**: Captured via `grim` to `docs/gpui/slice-06/evidence_slice06_wayland_workbench.png` verifying:
  - Row 1: Dominant full-width Search Input bar with magnifying glass icon and placeholder text (`Search packages and apps...`).
  - Row 2: Real popover trigger menus (`Filters ▾`, `State: All States ▾`, `Sort: Relevance ▾`) and view mode switcher (`[ Cards | Table ]`).
  - Active Filters: Dedicated chips with remove handlers and `Clear filters` action (preserving query and sort).
  - Failure Truth: Per-source health tracking, partial search retention with explicit warning and retry loops.

---

## Conclusion

All requirements of Slice-06R have been implemented, tested, built into an official Arch package, and verified live under native Wayland.
