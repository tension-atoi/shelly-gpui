# Shelly GPUI — Slice-04 Work Contract & Scope Boundaries

## 1. Purpose & Scope Boundaries

Slice-04 is a strictly bounded product polish and interaction slice.
Its purpose is to elevate the user experience through deterministic, GPU-accelerated motion, high-frequency invalidation isolation, pure settings rendering, and non-blocking toast feedback.

### Explicit Non-Goals
1. **No New Package Sources**: No additional package formats or sources are introduced.
2. **No Backend Architecture Redesign**: The Shelly Zig CLI (`Shelly.Cli.Zig`) and `ShellyClient` IPC contract remain untouched.
3. **No Unpinned Dependencies**: GPUI remains strictly pinned to `gpui = "0.2.2"`.
4. **No Deadcode or Suppression**: Zero `#[allow(dead_code)]` or warning suppressions are allowed.

---

## 2. Core Commitments & Deliverables

### 2.1 Pure Settings View Contract
- **Problem**: In previous iterations, toggling a setting immediately triggered disk writes (`ConfigManager::save_*`) during render callbacks or UI interactions without dirty tracking or batching.
- **Contract**:
  - `SettingsView` must operate as a pure render function.
  - State changes must mutate an in-memory draft (`SettingsDraft`).
  - An explicit `is_dirty()` flag must indicate uncommitted modifications.
  - Disk persistence is dispatched ONLY via an explicit user action ("Save Settings").
  - Upon successful persistence, an informational toast notification must be posted, and the dirty flag reset.

### 2.2 Backward-Compatible Motion Configuration
- **Contract**:
  - Add `reduce_motion: bool` to `GpuiUiConfig`.
  - Must use `#[serde(default)]` to guarantee 100% backward compatibility with legacy `gpui-ui.json` configuration files lacking this field.
  - Provide unit tests verifying both legacy deserialization (`reduce_motion == false`) and explicit deserialization (`reduce_motion == true`).
  - When `reduce_motion` is enabled, all continuous animations must snap immediately, and discrete transitions must render with full opacity without intermediate frames.

### 2.3 Motion System Contract
- **Contract**:
  - Standardized duration tokens:
    - `FAST`: 120ms (micro-interactions, opacity fades, tab changes).
    - `STANDARD`: 160ms (drawer disclosure, sidebar collapse/expand).
    - `EMPHASIS`: 220ms (critical error alerts, high-visibility state changes).
  - All continuous transitions must be interruptible and reversible from their current sampled values without abrupt visual jumping.
  - Hero headers (e.g. package inspector title and action buttons) must remain stationary during tab transitions.

### 2.4 High-Frequency UI Isolation Contract
- **Contract**:
  - Splitter dragging must NOT cause root `WorkspaceView` invalidations.
  - The workstation must be encapsulated in an independent child view (`PackageWorkstationView`).
  - Splitter width must be derived from pointer deltas:
    $$\text{width} = \text{clamp}(\text{start\_width} + (\text{current\_x} - \text{start\_x}), 280.0, 700.0)$$
  - Dragging must not glitch or jump when crossing sidebar collapse/expand thresholds.

### 2.5 Toast Feedback System Contract
- **Contract**:
  - Monotonic integer IDs.
  - Maximum 3 visible toasts on screen concurrently.
  - Deterministic FIFO eviction policy that strictly preserves error-severity toasts.
  - 3.5-second automatic decay timers.
  - Self-contained dismissal and optional action callbacks (`OpenLogs`).

---

## 3. Verification & Acceptance Requirements

1. **Compiler**: 0 errors, 0 warnings under `#![deny(dead_code, unused_variables, unused_imports, unused_must_use)]`.
2. **Clippy & Formatting**: Clean `cargo clippy -- -D warnings` and `cargo fmt --check`.
3. **Unit Tests**: 100% pass rate across all new and existing tests (minimum 45 tests).
4. **Wayland Framebuffer Evidence**: 6 empirical PNG captures from live Wayland compositor framebuffer verifying:
   - Sidebar expanded state.
   - Sidebar collapsed state (56px) with clean text clipping.
   - Splitter resized workstation.
   - Pure Settings view with draft dirty flag and save trigger.
   - Active toast feedback overlay.
   - Console log drawer disclosed at 220px.
