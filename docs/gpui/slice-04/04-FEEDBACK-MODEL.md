# Shelly GPUI — Slice-04 Feedback Model

## 1. Executive Summary

Feedback mechanisms in Shelly GPUI provide continuous, low-latency sensory verification of user actions, asynchronous system operations, and mutation outcomes. The feedback layer spans transient notifications (toasts), continuous progress indicators, clipboard telemetry, and configuration dirty-state tracking.

---

## 2. Notification Overlay System (`ToastCenter` & `ToastOverlay`)

### 2.1 State Architecture & Capacity Bounds
- **Monotonic ID Generation**: Every toast receives a strictly increasing `u64` identifier (`next_id += 1`).
- **Bounded Concurrency**: A hard maximum of 3 concurrent visible toasts (`TOAST_MAX_VISIBLE = 3`).
- **Deterministic Eviction**:
  - When a 4th toast arrives, `ToastCenter` evicts the oldest non-error notification:
    ```rust
    let evict_idx = self
        .toasts
        .iter()
        .position(|t| t.kind != ToastKind::Error)
        .unwrap_or(0);
    self.toasts.remove(evict_idx);
    ```
  - High-severity errors remain visible until either explicitly dismissed by the user or timed out naturally.

### 2.2 Lifecycle & GPUI Animation Binding
Toasts progress through three distinct states:
1. **`Entering`**:
   - Duration: 120 ms (`MotionDurations::FAST`).
   - Motion: Rendered with GPUI `with_animation`:
     ```rust
     toast_el.with_animation(
         ElementId::NamedInteger("toast-enter".into(), toast_id),
         Animation::new(MotionDurations::FAST).with_easing(gpui::ease_out_quint()),
         |el, delta| el.opacity(delta),
     )
     ```
2. **`Visible`**:
   - Duration: 3500 ms (`TOAST_DISPLAY_DURATION`).
   - Rendered at static opacity `1.0`.
3. **`Exiting`**:
   - Duration: 120 ms (`MotionDurations::FAST`).
   - Motion: Rendered with GPUI `with_animation`:
     ```rust
     toast_el.with_animation(
         ElementId::NamedInteger("toast-exit".into(), toast_id),
         Animation::new(MotionDurations::FAST).with_easing(gpui::ease_out_quint()),
         |el, delta| el.opacity(1.0 - delta),
     )
     ```
   - On completion of the 120ms exit timer, the toast entity is removed from memory.

### 2.3 Reduced Motion Invariance
When `reduce_motion = true`:
- New toasts transition immediately to `ToastLifecycle::Visible` (0 ms entrance).
- Animated wrappers are bypassed (`toast_el.opacity(1.0)`).
- Manual dismissal via the close button (`✕`) immediately removes the toast without exit delay.

### 2.4 Actionable Feedback
- Error toasts support contextual actions, such as `ToastAction::OpenLogs`.
- Activating the action button automatically triggers console drawer disclosure, allowing immediate inspection of compilation or network failure logs.

---

## 3. Search & Query Telemetry

1. **Async Query Activity**:
   - When background search queries are in-flight (`is_searching = true`), a pulsing status badge displays: `⚡ Searching...`.
   - Normal motion: Uses GPUI continuous animation with `pulsating_between(0.4, 1.0)` over 800ms.
   - Reduced motion: Renders as a crisp, static indicator at full opacity.
2. **Result Count Badges**:
   - On query resolution, the active results count is displayed (e.g. `24 results`), confirming the scope of available packages.

---

## 4. Clipboard & System Operations Telemetry

1. **Clipboard Confirmation**:
   - Clicking "Copier" in the operations console copies raw terminal output to the system clipboard via `cx.write_to_clipboard(...)`.
   - Provides dual feedback:
     - Visual badge change on the copy button with a 2-second decay timer.
     - Transient informational toast (`ToastKind::Info`: "Journaux copiés dans le presse-papiers").
2. **Operation Status Pills**:
   - Running operations render an active status indicator with the operation name.
   - Success and error states color the log drawer header and emit matching toasts.

---

## 5. Configuration Dirty-State Feedback

In `SettingsView`:
1. **Unsaved Changes Pill**:
   - Clean state: Renders muted `"À jour"` pill.
   - Dirty state (`is_dirty = true`): Renders prominent warning badge `"Modifications non enregistrées"`.
2. **Save Button State**:
   - When clean: Rendered with inactive background, muted typography, no cursor pointer, no hover state, and no click handler attached.
   - When dirty: Rendered with active accent color, bold text (`"Enregistrer les paramètres *"`), pointer cursor, hover highlight, and active click handler dispatching disk write.
3. **Live Reduce Motion Verification**:
   - Toggling the Reduce Motion switch immediately alters runtime animation policies across all open views without requiring prior disk persistence.
