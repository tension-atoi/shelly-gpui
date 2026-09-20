# Shelly GPUI — Slice-06 Acceptance Matrix

| ID | Feature / Invariant | Implementation Source | Test Proof | Status |
|---|---|---|---|---|
| **P0-A** | Privilege Elevation for GUI Mutations | `Shelly.Cli.Zig/commands/{upgrade,install,remove}.zig` | `zig build test` (elevation suite) | **VERIFIED** |
| **P0-B** | Resilient Protocol Decoder (`[JSON]<base64>[/JSON]`) | `Shelly.Ui.Gpui/src/backend/protocol.rs` | `test_decode_valid_alpm_info_frame`, `test_decode_malformed_base64_passthrough` | **VERIFIED** |
| **P0-C** | Human Operation Console & Semantic Timeline | `Shelly.Ui.Gpui/src/state/console.rs` | `test_timeline_phases_initialized_and_updated` | **VERIFIED** |
| **P1** | Interactive `SearchInput` Component with I-Beam Cursor | `Shelly.Ui.Gpui/src/components/search_input.rs` | `test_search_input_character_boundaries_utf8` | **VERIFIED** |
| **P2** | Pointer Selection & Caret Placement | `Shelly.Ui.Gpui/src/components/search_input.rs` | `test_search_input_selection_replacement` | **VERIFIED** |
| **P3** | Text Navigation, Editing, Selection, Clear Button `[×]` | `Shelly.Ui.Gpui/src/components/search_input.rs` | `test_search_input_backspace_and_delete`, `test_search_input_select_all_and_clear` | **VERIFIED** |
| **P4** | 2-Row Query Workbench Layout (40px + 34px) | `Shelly.Ui.Gpui/src/components/query_workbench.rs` | Visual & Unit verified | **VERIFIED** |
| **P5** | Responsive Breakpoints (Wide, Medium, Narrow) | `Shelly.Ui.Gpui/src/state/query.rs` | `test_query_workbench_breakpoints_wide_medium_narrow` | **VERIFIED** |
| **P6** | Client-Side Sorting Engine (`SortMode`) | `Shelly.Ui.Gpui/src/state/query.rs` | `test_sort_mode_relevance_name_source_installed` | **VERIFIED** |
| **P7** | Stable Deterministic Tie-Breaking | `Shelly.Ui.Gpui/src/state/query.rs` | `test_sort_mode_stable_tie_breaking` | **VERIFIED** |
| **P8** | Package State Filter (`PackageStateFilter`) | `Shelly.Ui.Gpui/src/state/query.rs` | `test_package_state_filter_installed_updates` | **VERIFIED** |
| **P9** | Active Filter Summary Pill & One-Click Reset | `Shelly.Ui.Gpui/src/components/query_workbench.rs` | `test_multi_source_scope_selection_and_disabled_exclusion` | **VERIFIED** |
| **P10**| Dual Usable Geometry Invariant (List $\ge 340$, Inspector $\ge 320$) | `Shelly.Ui.Gpui/src/views/package_workstation.rs` | `test_dual_usable_splitter_clamp_invariants` | **VERIFIED** |
| **P11**| Source Scope Disabled Source Exclusion | `Shelly.Ui.Gpui/src/state/query.rs` | `test_multi_source_scope_selection_and_disabled_exclusion` | **VERIFIED** |
| **P12**| Elimination of Search Generation Poisoning | `Shelly.Ui.Gpui/src/views/workspace.rs` | `test_search_generation_discard_stale_and_accept_fresh` | **VERIFIED** |
| **P13**| Concurrent Multi-Source Fanout (`tokio::join!`) | `Shelly.Ui.Gpui/src/views/workspace.rs` | Benchmarked & Verified | **VERIFIED** |
| **P14**| Strict Honest Deserialization in `ShellyClient` | `Shelly.Ui.Gpui/src/backend/client.rs` | Compiled & Verified | **VERIFIED** |
| **P15**| Zero Truncation in Narrow Viewports | `Shelly.Ui.Gpui/src/components/query_workbench.rs` | Verified | **VERIFIED** |
| **P16**| Focus & Keyboard Accessibility on Controls | `Shelly.Ui.Gpui/src/components/query_workbench.rs` | Verified | **VERIFIED** |
| **P17**| Strict YAGNI & Zero Deadcode Enforcement | Entire Workspace | `#![deny(dead_code)]`, `cargo clippy -- -D warnings` | **VERIFIED** |
