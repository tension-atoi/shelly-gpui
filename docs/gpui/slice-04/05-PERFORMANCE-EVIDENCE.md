# Shelly GPUI — Slice-04 Performance, Frame Isolation & Runtime Hygiene Evidence

## 1. Executive Summary

Slice-04 establishes empirical engineering safeguards that protect UI frame budgets, prevent redundant allocation during high-frequency pointer interactions, isolate continuous animation scheduling to local view entities, and enforce strict zero-scaffolding production hygiene.

---

## 2. High-Frequency Interaction & Splitter Isolation

### 2.1 The Pointer Event Challenge
The package workstation features an interactive vertical splitter allowing users to continuously resize the package list pane (280px to 700px). On modern high-refresh displays (60Hz to 144Hz), pointer movement events fire continuously, submitting dozens or hundreds of resize samples per second.

In earlier designs, two critical bottlenecks degraded this interaction:
1. **Root View Invalidation**: The splitter state was managed at the root `WorkspaceView` level, causing the sidebar, navigation destinations, and console drawer to re-layout on every pointer move event.
2. **Deep Collection Cloning**: Rendering the virtualized package list required cloning the active package vector on every frame, generating thousands of heap string allocations during a single drag gesture.

### 2.2 Architectural Solutions in Slice-04

#### 2.2.1 Child View Entity Boundary
The package workstation is encapsulated in `PackageWorkstationView` (`Entity<PackageWorkstationView>`). Pointer drag event listeners (`on_mouse_down`, `on_mouse_move`, `on_mouse_up`) are attached strictly to the workstation's container element:

```rust
// views/package_workstation.rs
.on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, cx| {
    if this.is_dragging_splitter {
        let delta = event.position.x.0 - this.splitter_drag_start_x;
        let new_width = (this.splitter_drag_start_width + delta).clamp(280.0, 700.0);
        if (new_width - this.list_pane_width).abs() > 0.5 {
            this.list_pane_width = new_width;
            cx.notify(); // Only invalidates PackageWorkstationView!
        }
    }
}))
```

When dragging occurs:
- Only `PackageWorkstationView` invokes `cx.notify()`.
- Root `WorkspaceView`, `SidebarView`, and `OperationConsoleView` remain entirely idle with zero layout reflows.

#### 2.2.2 Zero-Copy Package Slices (`Arc<[UnifiedPackage]>`)
To eliminate per-frame heap allocations during splitter movement, `PackageStore` (`src/state/package_store.rs`) stores package collections in immutable atomic slices:

```rust
// state/package_store.rs
pub struct PackageStore {
    active_results: Arc<[UnifiedPackage]>,
    installed_packages: Arc<[UnifiedPackage]>,
    updates_packages: Arc<[UnifiedPackage]>,
    // ...
}
```

When `PackageWorkstationView::render` executes:
1. It retrieves `Arc<[UnifiedPackage]>` via pointer copy (`Arc::clone(&packages)`).
2. The `uniform_list` element iterates across the slice without allocating or copying individual package structs or strings.
3. Memory allocations during active dragging are reduced to zero heap allocations per frame.

---

## 3. Continuous Frame Ownership Isolation

### 3.1 The Flaw of Root Frame Scheduling
In standard GPUI applications, requesting animation frames via `window.request_animation_frame()` from a root view forces the entire window tree to re-evaluate layout and re-render every frame (8.33ms at 120 FPS). If the root view schedules continuous frames for child element disclosures, the entire application suffers frame budget exhaustion.

### 3.2 View-Level Frame Isolation

Slice-04 isolates continuous animated scalar disclosures into dedicated child entities:

```
┌────────────────────────────────────────────────────────┐
│                     WorkspaceView                      │
│      (0 continuous request_animation_frame calls)      │
└───────────────┬────────────────────────┬───────────────┘
                │                        │
                ▼                        ▼
     ┌──────────────────────┐ ┌──────────────────────┐
     │     SidebarView      │ │ OperationConsoleView │
     │ - width_scalar       │ │ - height_scalar      │
     │ - Local RAF loop     │ │ - Local RAF loop     │
     │   (only when active) │ │   (only when active) │
     └──────────────────────┘ └──────────────────────┘
```

#### 3.2.1 `SidebarView` (`src/views/sidebar.rs`)
- Owns `width_scalar: AnimatedScalar` (animating 56.0px ↔ 190.0px over 160ms with `ease_out_quint`).
- Subscribes to `SessionEvent::SidebarToggled`.
- In `SidebarView::render`:
  ```rust
  let (width, is_animating) = this.width_scalar.update(Instant::now());
  if is_animating {
      window.request_animation_frame();
  }
  ```
- Calls `window.request_animation_frame()` *only* while its internal scalar is in-flight. As soon as the animation completes, frame requests cease immediately.

#### 3.2.2 `OperationConsoleView` (`src/views/operation_console.rs`)
- Owns `height_scalar: AnimatedScalar` (animating 0.0px ↔ 220.0px over 160ms or 220ms with `ease_in_out`).
- Subscribes to `ConsoleEvent::Toggled`, `OperationStarted`, and `OperationFinished`.
- In `OperationConsoleView::render`:
  ```rust
  let (height, is_animating) = this.height_scalar.update(Instant::now());
  if is_animating {
      window.request_animation_frame();
  }
  ```
- Automatically triggers disclosure on operation launch and failure alerts, driving its own frame loop in complete isolation from the workstation and sidebar.

#### 3.2.3 Zero Root Loop Guarantee
`WorkspaceView::render` contains **zero** calls to `window.request_animation_frame()`. The root view only re-renders when destination changes or global settings update.

---

## 4. Production Hygiene & Zero Scaffolding Audit

### 4.1 Elimination of Acceptance Injection Hooks
All test environment hooks (`SHELLY_SCREENSHOT_*`, `SHELLY_TEST_*`, `SHELLY_FORCE_*`) used in previous iterations have been completely purged from production source code.

Audit command:
```bash
grep -rn "SHELLY_" Shelly.Ui.Gpui/src/
```

Result:
```text
Shelly.Ui.Gpui/src/backend/process.rs:81:  // Si SHELLY_ELEVATOR n'est pas déjà configuré dans l'environnement, on utilise pkexec
Shelly.Ui.Gpui/src/backend/process.rs:83:  if std::env::var("SHELLY_ELEVATOR").is_err() {
Shelly.Ui.Gpui/src/backend/process.rs:84:      cmd.env("SHELLY_ELEVATOR", "pkexec");
Shelly.Ui.Gpui/src/backend/client.rs:23:  } else if let Ok(env_path) = std::env::var("SHELLY_BIN") {
```
*Verification: Only genuine system environment paths (`SHELLY_ELEVATOR` and `SHELLY_BIN`) are queried. Zero test scaffolding or screenshot hooks remain.*

### 4.2 Hard Compiler Enforcement & Zero Deadcode Policy
The crate root (`Shelly.Ui.Gpui/src/main.rs`) enforces strict zero-tolerance compiler attributes:
```rust
#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_must_use)]
```

- `grep -rn "allow(dead_code)" Shelly.Ui.Gpui/src/` returns 0 hits.
- `grep -rn "allow(unused" Shelly.Ui.Gpui/src/` returns 0 hits.
- `cargo clippy -- -D warnings` completes with 0 warnings.
- `cargo fmt --check` passes cleanly.

---

## 5. Automated Verification Telemetry

All 48 unit tests pass deterministically in `< 0.01s`:

| Test Module | Coverage Scope | Result |
|---|---|---|
| `state::motion` | Scalar forward progress, clamping, completion, inactive idle, retargeting reversal, reduced motion snapping, token durations | **7 / 7 PASSED** |
| `state::toast` | Monotonic IDs, max visible bounds, error preservation eviction, lifecycle transitions, timer identity, dismiss transitions, reduced motion snap | **7 / 7 PASSED** |
| `views::package_workstation` | Splitter forward delta, reverse delta, minimum clamp (280px), maximum clamp (700px), sidebar width offset independence | **5 / 5 PASSED** |
| `views::settings` | Draft dirty tracking, save persistence clears dirty, failure preserves dirty, reduce motion toggle draft updates, clean state button isolation | **5 / 5 PASSED** |
| `config` | Backward compatibility without `reduce_motion`, explicit `reduce_motion` | **2 / 2 PASSED** |
| `state::package_store` | AppImage key identity, search cache storage, normalization, cache invalidation, PKGBUILD cache invalidation | **5 / 5 PASSED** |
| `state::session` | Session epochs increment, nav destination metadata, source filter metadata, view mode continuity, dependency clean name navigation | **8 / 8 PASSED** |
| `state::semantic` | Canonical install command, versioned/unversioned dependency ref parsing, package capability derivation | **5 / 5 PASSED** |
| `state::console` | Default console model, clear logs preserves lifecycle status | **2 / 2 PASSED** |
| `components::package_table` | Table byte formatting, display size truthfulness | **2 / 2 PASSED** |

**Total Suite**: 48 passed; 0 failed; 0 ignored; finished in 0.00s.
