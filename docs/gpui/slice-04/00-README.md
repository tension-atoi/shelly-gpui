# Shelly GPUI — Slice-04 Documentation Kit

## Target Slice

**SLICE-04 — Motion, Interaction Polish, Feedback & High-Frequency UI Isolation**

This slice follows the Slice-03 workstation foundation (`PackageViewMode::Table`, `PackageViewMode::Cards`, decomposed 3-tab inspector, `DependencyRef`, capability derivation) and delivers a high-polish, GPU-accelerated interaction layer.

It purposefully delivers:

1. **Pure Settings View & Draft State (`SettingsView`)**:
   - Complete removal of render-time disk mutations (`save_*` calls during rendering).
   - Pure, in-memory draft configuration state (`SettingsDraft`) with dirty tracking (`is_dirty()`).
   - Explicit "Save Settings" dispatch with persistent storage via `ConfigManager` and immediate toast feedback.
2. **Backward-Compatible Motion Configuration**:
   - `reduce_motion: bool` integrated into `GpuiUiConfig` with `#[serde(default)]` and robust JSON compatibility.
   - Complete support for user preference toggling with instant motion suppression.
3. **Formalized Motion System & Primitives (`state::motion`)**:
   - Standardized motion token durations: `FAST = 120ms`, `STANDARD = 160ms`, `EMPHASIS = 220ms`.
   - Continuous, interruptible, reversible transitions via `AnimatedScalar` with quintic (`ease_out_quint`) and sinusoidal (`ease_in_out`) easings.
   - Discrete element transitions: 120ms opacity fade for destinations and inspector tabs (`with_animation`), with stationary hero header.
4. **High-Frequency UI Invalidation Isolation**:
   - Encapsulation of list pane, splitter, and inspector into a dedicated `PackageWorkstationView`.
   - Pure pointer-delta drag math: `(start_width + (current_x - start_x)).clamp(280.0, 700.0)`.
   - Splitter dragging and hover states strictly invalidate only the child workstation, protecting root `WorkspaceView` and sidebar from unnecessary layout reflows.
5. **Continuous Sidebar & Console Drawer Disclosure**:
   - Sidebar continuous width transition (56px ↔ 190px) with text clipping (`overflow_hidden()`) preventing label wrapping/bleed.
   - Console log drawer continuous height transition (0px ↔ 220px) with clean scroll viewport clipping and interruptible reversal.
6. **Bounded Toast Center & Feedback Overlay (`ToastCenter` & `ToastOverlay`)**:
   - Monotonic IDs, bounded maximum of 3 concurrent visible notifications.
   - Deterministic FIFO eviction policy strictly preserving high-severity error notifications.
   - Self-contained 3.5-second decay timers with interactive actions (e.g. `OpenLogs`).
7. **Strict Compiler Enforcement & Zero Deadcode**:
   - Hard `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, `#![deny(unused_must_use)]`.
   - 0 warnings, pinned GPUI 0.2.2 compatibility.

---

## Repository Baseline

Repository:

    tension-atoi/shelly-gpui

Canonical branch:

    main

Frontend dependency:

    gpui = "0.2.2"

Functional upstream authority:

    Seafoam-Labs/Shelly-ALPM

---

## Documentation Kit Map

- [`00-README.md`](00-README.md): High-level slice scope, primitives, and repository baseline.
- [`01-CURRENT-STATE.md`](01-CURRENT-STATE.md): Architectural snapshot and component inventory.
- [`02-WORK-CONTRACT.md`](02-WORK-CONTRACT.md): Detailed work contract commitments, constraints, and scope boundaries.
- [`03-MOTION-SPECIFICATION.md`](03-MOTION-SPECIFICATION.md): Mathematical motion specification, easing curves, and timing policies.
- [`04-ACCEPTANCE-MATRIX.md`](04-ACCEPTANCE-MATRIX.md): Verification matrix mapping specifications to evidence classes.
- [`05-ACCEPTANCE-REPORT.md`](05-ACCEPTANCE-REPORT.md): Final acceptance report with empirical test telemetry and native Wayland runtime evidence.
