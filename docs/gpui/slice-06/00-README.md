# Shelly GPUI — Slice-06 Documentation Kit

## Target Slice

**SLICE-06 — Search Correctness, Query Workbench & Privileged Operations**

This slice fundamentally elevates Shelly GPUI's query execution layer, search UX, and backend privilege handling. It resolves critical deficiencies where privileged operations previously failed under GUI mode, replaces synthetic search UI with an interactive native GPUI text editing input, introduces a responsive 2-row Query Workbench with client-side sorting and state filters, guarantees dual usable geometry for workstation panes, establishes a structured base64 IPC protocol with semantic timeline tracking, and eliminates asynchronous search generation poisoning.

---

## Core Architectural Deliverables

1. **Privilege Elevation under UI Mode (`Shelly.Cli.Zig`)**:
   - In `upgrade.zig`, `install.zig`, and `remove.zig`, non-root ALPM mutations invoked under `--ui-mode` automatically invoke `pkexec` privilege elevation, preserving structured log streaming back to GPUI.
   - User-scoped package targets (e.g. `flatpak install --user`) correctly bypass elevation.
   - Comprehensive unit tests in Zig CLI verify elevation invocation under UI mode.

2. **Structured IPC Protocol & Human Console Timeline (`backend/protocol.rs`, `state/console.rs`)**:
   - Bounded, resilient decoder for structured `[JSON]<base64>[/JSON]` execution frames emitted by the CLI.
   - Strongly typed `UiFrame` models: `AlpmInfo`, `AlpmError`, `OperationProgress`, `TransactionEvent`.
   - `ConsoleModel` maintains an operation timeline tracking backend phases (Standard, AUR, Flatpak, AppImage) alongside raw stdout/stderr logs.
   - Structured error details surface domain-level diagnostics without truncating technical information.

3. **Canonical Interactive `SearchInput` Component (`components/search_input.rs`)**:
   - Native GPUI text input implementing `EntityInputHandler` and `ElementInputHandler`.
   - Full pointer caret positioning, text drag selection, and I-beam cursor styling.
   - Comprehensive keyboard navigation: typing, Backspace/Delete with selection support, Left/Right/Home/End cursor navigation, Shift selection extension, Ctrl+A select-all, and Escape quick-clear.
   - Integrated clear button (`[×]`) and search activity pulsing indicator respecting `reduce_motion`.

4. **Responsive 2-Row Query Workbench (`components/query_workbench.rs`)**:
   - Clean separation into Row 1 (prominent `SearchInput` at 40px) and Row 2 (query controls at 34px).
   - Three responsive layout breakpoints based on list pane width:
     - **Wide ($\ge 540\text{px}$)**: Full source pills (`All`, `ALPM`, `AUR`, `Flatpak`, `AppImage`), State filter button, Sort mode button, View mode switcher, and expanded Active Filter Summary pill.
     - **Medium ($380\text{px} \le w < 540\text{px}$)**: Compact pill labels (`All`, `ALPM`, `AUR`, `FP`, `AI`), cycling State filter, Sort mode, View mode switcher.
     - **Narrow ($< 380\text{px}$)**: Compact cycling source selector, Sort mode, View mode switcher, zero text clipping.
   - Active filter summary with one-click "Reset Filters" action.

5. **Client-Side Sorting & State Filtering (`state/query.rs`)**:
   - `SortMode`: 6 deterministic modes (`Relevance`, `NameAsc`, `NameDesc`, `Source`, `InstalledFirst`, `UpdatesFirst`) with stable secondary and tertiary tie-breaking.
   - `PackageStateFilter`: 4 states (`All`, `Installed`, `NotInstalled`, `UpdatesAvailable`).
   - Zero backend re-querying: sorting and state filters operate strictly client-side on loaded results.

6. **Dual Usable Geometry Invariant (`views/package_workstation.rs`, `ui_metrics.rs`)**:
   - `LIST_MIN_USABLE = 340.0px` and `INSPECTOR_MIN_USABLE = 320.0px`.
   - Clamping logic guarantees that neither the list pane nor the inspector can be squeezed below their usable thresholds regardless of viewport size.

7. **Search Generation Poisoning Elimination & Concurrent Multi-Source Fanout (`views/workspace.rs`, `state/package_store.rs`, `backend/client.rs`)**:
   - Replaced `usize::MAX` generation reset with monotonic `next_search_generation()`, completely preventing subsequent search queries from being poisoned or discarded.
   - Multi-source search (`SourceFilter::All`) executes concurrent fanout across standard ALPM, AUR, Flatpak, and AppImage backends via `tokio::join!`.
   - Honest backend deserialization in `ShellyClient`: malformed output returns typed `Result::Err` with context rather than silently hiding failures.

8. **Strict Compiler Standards & Packaging Verification**:
   - Hard compiler enforcement: `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, `#![deny(unused_must_use)]`.
   - 88 unit tests passing with zero failures and zero warning suppressions.
   - Native Arch package built from remote HEAD and verified running under Wayland.
