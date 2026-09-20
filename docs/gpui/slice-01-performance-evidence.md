# Slice-01 Performance & Runtime Evidence Record

**Date**: 2026-09-20  
**Baseline commit**: `e43b03ffa4aaab5d99a151d6a2491370e1f65161`  
**Frontend Engine**: GPUI 0.2.2 (`crates.io`)  
**Target Binary**: `shelly-gpui` (Release & Debug profiles)

This document provides a disciplined, evidence-based record of the runtime and performance characteristics established during Slice 01, strictly distinguishing between measured data, observed behavior, established architecture, and inference.

---

## 1. Measurement Terminology

- **[MEASURED]**: Directly captured using system profiling tools (`/proc`, `time`, binary inspect).
- **[OBSERVED]**: Empirically confirmed during active interactive execution on Wayland.
- **[ESTABLISHED]**: Verified directly in source code and compiler diagnostics.
- **[INFERENCE]**: Plausible deduction based on architecture, not yet backed by an instrumented profiler probe.

---

## 2. Package List Virtualization

- **[ESTABLISHED]**: The package list construction was migrated from unbounded child rendering to `gpui::uniform_list`.
- **[ESTABLISHED]**: Item geometry is fixed at `78.0 px` height per row item, tracked via `UniformListScrollHandle`.
- **[MEASURED]**: Test package collection size: 1,482 packages loaded in local ALPM inventory.
- **[OBSERVED]**: Rapid vertical scrolling through 1,400+ packages renders without stutter or frame stalls under Wayland compositor.
- **[INFERENCE]**: Frame budget remains comfortably below 16.6ms (60 Hz) / 8.33ms (120 Hz) due to viewport culling (only ~12-15 item cards instantiated per frame instead of 1,482).

---

## 3. Package Detail Cache

- **[ESTABLISHED]**: In-memory detail cache implemented as `HashMap<String, AlpmPackage>` inside the frontend.
- **[OBSERVED]**: Re-selecting an already inspected package displays details immediately without spawning a duplicate `shelly list alpm -j` child process.
- **[ESTABLISHED]**: Cache currently lives inside the root `WorkspaceView`, tightly coupling domain data with UI presentation. (Targeted for migration in Slice 02).

---

## 4. Splitter & Resize Invalidation

- **[ESTABLISHED]**: Splitter drag implementation discards sub-pixel mouse movement `< 1.0 px`.
- **[OBSERVED]**: Splitter dragging is responsive and visually smooth.
- **[ESTABLISHED] [LIMITATION]**: The splitter drag still updates `WorkspaceView::list_pane_width` and triggers `cx.notify()` on the root view. The root invalidation boundary has **not** been eliminated; the entire workspace view re-renders during drag.

---

## 5. Console & Operation Logs

- **[ESTABLISHED]**: Typed `LogEntry` records distinguishing stdout from stderr.
- **[ESTABLISHED]**: ANSI escape code sanitization cleans raw terminal output.
- **[ESTABLISHED]**: Dedicated interactive controls: Copy Logs to clipboard, Clear buffer, Drawer expand/collapse toggle.
- **[ESTABLISHED] [LIMITATION]**: The "Auto-scroll" button toggles a boolean state `auto_scroll_logs`, but the `LogDrawer` child container (`div().overflow_scroll()`) does not possess a bound `ScrollHandle` or viewport auto-tracking mechanism. Auto-scrolling is an active UI state, but functional viewport auto-follow is **not yet established**.

---

## 6. Privilege Elevation

- **[ESTABLISHED]**: Streaming package mutations set `SHELLY_ELEVATOR=pkexec` when not otherwise configured in the user environment.
- **[OBSERVED]**: Polkit graphical authentication prompt appears cleanly when running install/remove operations without root.

---

## 7. Build Metrics

- **[MEASURED]**: Binary size (release, stripped, thin LTO): **20 MB** (`/mnt/workbench/target/release/shelly-gpui`).
- **[MEASURED]**: Incremental dev build time: **~0.25s - 1.2s**.
- **[MEASURED]**: Clean release build time: **20.12s**.
- **[MEASURED]**: Compiler diagnostics: **0 errors, 0 warnings** under `#![deny(dead_code, unused_variables, unused_imports, unused_must_use)]`.

---

## 8. Summary of Known Limitations

1. **Auto-Scroll Gap**: `LogDrawer` requires explicit GPUI `ScrollHandle` tracking to actually follow newest streaming lines.
2. **State Monolith**: `WorkspaceView` still owns package catalog, search state, console buffer, and layout dimensions.
3. **Splitter Re-render Scope**: Splitter resizing invalidates root view rather than an isolated layout container.
4. **Navigation/Filter Conflation**: Top horizontal tabs mix package sources (`ALPM`, `AUR`, `Flatpak`) with destinations (`Updates`, `News`, `Settings`).
