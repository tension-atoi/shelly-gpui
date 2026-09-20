# Shelly GPUI — Slice-04 Motion Architecture & Mathematical Model

## 1. Design Philosophy & Motion Tokens

Shelly GPUI implements a deterministic, GPU-accelerated motion architecture built on GPUI 0.2.2 primitives. Motion in Shelly is purposeful and functional: orienting the user during navigation transitions, conveying interface hierarchy, and delivering responsive spatial manipulation without hindering high-throughput package management workflows.

### 1.1 Standardized Duration Tokens

All dynamic transitions throughout the application are strictly bounded by three duration tokens defined in `src/state/motion.rs`:

| Token | Duration | Scope of Application |
|---|---|---|
| `FAST` | **120 ms** | Discrete navigation entrance fades, inspector tab transitions, toast entrance and exit fades. |
| `STANDARD` | **160 ms** | Continuous layout disclosure: sidebar collapse/expand, normal console drawer disclosure. |
| `EMPHASIS` | **220 ms** | High-severity alerts, operation failure console drawer automatic disclosure. |

---

## 2. Mathematical Easing Curves

All easing curves are pure mathematical functions $f: [0, 1] \to [0, 1]$ satisfying $f(0) = 0$ and $f(1) = 1$.

### 2.1 Quintic Ease-Out (`ease_out_quint`)
Used for element entrances, interactive adjustments, and toast fade transitions. Delivers immediate initial velocity followed by smooth deceleration into rest:

$$f(t) = 1 - (1 - t)^5$$

Implementation in `src/state/motion.rs`:
```rust
pub fn ease_out_quint(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(5)
}
```

### 2.2 Piecewise Quadratic Ease-In-Out (`ease_in_out`)
Used for continuous reversible spatial disclosures (such as the operations console drawer) where symmetric acceleration and deceleration provide natural physical weight without overshoot:

$$f(t) = \begin{cases} 2t^2 & \text{if } t < 0.5 \\ 1 - 2(1 - t)^2 = -1 + 4t - 2t^2 & \text{if } t \ge 0.5 \end{cases}$$

Implementation in `src/state/motion.rs`:
```rust
pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        let x = -2.0 * t + 2.0;
        1.0 - x * x / 2.0
    }
}
```

---

## 3. Continuous Transitions: The `AnimatedScalar` Model & Frame Isolation

Continuous spatial dimensions (sidebar width: 56px ↔ 190px, console height: 0px ↔ 220px) are governed by the `AnimatedScalar` state machine.

### 3.1 State Representation & Progress Mechanics
```rust
pub struct AnimatedScalar {
    pub current: f32,
    pub start_value: f32,
    pub target_value: f32,
    pub start_time: Option<Instant>,
    pub duration: Duration,
    pub easing: fn(f32) -> f32,
}
```

At any frame rendered at timestamp $T_{\text{now}}$:
1. $\Delta t = \text{saturating\_sub}(T_{\text{now}}, T_{\text{start}})$
2. Normalized progress $p = \text{clamp}\left(\frac{\Delta t}{D}, 0.0, 1.0\right)$
3. Instantaneous value: $v_{\text{current}} = v_{\text{start}} + (v_{\text{target}} - v_{\text{start}}) \cdot f(p)$

### 3.2 Mid-Flight Reversal (Retargeting)
When an ongoing transition is interrupted (e.g. user triggers sidebar toggle while mid-collapse):
- `retarget()` samples the instantaneous $v_{\text{current}}$, assigns $v_{\text{start}} = v_{\text{current}}$, sets the new target, and anchors the start time to `now`.
- This ensures $C^0$ positional continuity with zero geometric jumping or visual popping.

### 3.3 Continuous Frame Ownership Isolation
To prevent continuous animations from forcing re-renders across the entire root workspace at 60/120 FPS:
- **`SidebarView` (`src/views/sidebar.rs`)**: Encapsulates `sidebar_width_scalar`. Subscribes to `SessionEvent::SidebarToggled`. Requests animation frames via `window.request_animation_frame()` *only* while its scalar is active.
- **`OperationConsoleView` (`src/views/operation_console.rs`)**: Encapsulates `console_height_scalar`. Subscribes to `ConsoleEvent::Toggled`, `OperationStarted`, and `OperationFinished`. Requests animation frames *only* while its scalar is active.
- **`WorkspaceView`**: Owns zero continuous scalar loops and requests zero frames in its `render()` function.

---

## 4. Discrete Transitions: GPUI `with_animation`

Discrete transitions occur when navigation or tab states switch.

### 4.1 Invariant: Elements Must Precede Animation
In GPUI 0.2.2, `AnimationElement<Self>` implements `IntoElement` but does not expose `.child(...)`. Therefore, child elements are fully attached to the container before applying `.with_animation()`:

```rust
div()
    .id("dest_view")
    .size_full()
    .child(main_content)
    .with_animation(
        ("dest-fade", destination_epoch as usize),
        Animation::new(MotionDurations::FAST).with_easing(gpui::ease_out_quint()),
        |elem, delta| elem.opacity(delta),
    )
```

### 4.2 Entrance Fade Terminology
Discrete transitions implement an incoming **entrance fade** (`opacity: 0.0 -> 1.0` over 120ms). Incoming views smoothly phase in over the outgoing canvas without layout jitter.

### 4.3 Stationary Inspector Hero Header Contract
When switching inspector tabs (Overview ↔ Dependencies ↔ Files & Build), the `InspectorHeader` hero section (title, version, status pill, actions, and tab bar) remains stationary outside the animated viewport. Only the sub-tab body undergoes the 120ms entrance fade.

---

## 5. Reduced Motion Policy (`MotionPolicy`)

Shelly respects user motion preferences via `GpuiUiConfig.reduce_motion`:
1. **Immediate Snapping**: `AnimatedScalar::retarget()` immediately invokes `.snap(new_target)`, converging in 0 frames.
2. **Animation Bypass**: Discrete element transitions bypass `.with_animation(...)` and render at static opacity 1.0.
3. **Toast Snapping**: Toasts render at full opacity immediately on post and dismiss instantaneously without exiting fade.
4. **Live Synchronization**: Toggling Reduce Motion in `SettingsView` immediately updates `WorkspaceView`, `PackageWorkstationView`, `SidebarView`, and `OperationConsoleView`, snapping any in-flight transitions without requiring disk save.
