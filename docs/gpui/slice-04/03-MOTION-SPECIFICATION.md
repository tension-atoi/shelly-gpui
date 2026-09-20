# Shelly GPUI — Slice-04 Motion Specification & Mathematical Model

## 1. Design Philosophy & Motion Tokens

Shelly GPUI implements a deterministic, GPU-driven motion architecture built on GPUI 0.2.2 primitives. Motion in Shelly serves functional clarity: orienting the user during context switches, conveying hierarchy, and providing smooth mechanical feel without slowing down expert package management workflows.

### 1.1 Standardized Duration Tokens

All transitions in the application are bounded by three canonical duration tokens:

| Token | Duration | Scope of Application |
|---|---|---|
| `FAST` | **120 ms** | Micro-interactions, discrete view cross-fades, tab content entrance, toast dismissals. |
| `STANDARD` | **160 ms** | Continuous layout adjustments: sidebar collapse/expand, normal console disclosure. |
| `EMPHASIS` | **220 ms** | High-severity alerts, operation failure console disclosures, critical toast appearances. |

---

## 2. Mathematical Easing Curves

Motion curves are expressed as pure functions $f: [0, 1] \to [0, 1]$.

### 2.1 Quintic Ease-Out (`ease_out_quint`)
Used for element entrances and interactive adjustments. It delivers an immediate initial velocity followed by a gradual, smooth deceleration into rest.

$$f(t) = 1 - (1 - t)^5$$

Implementation in `src/state/motion.rs`:
```rust
pub fn ease_out_quint(t: f32) -> f32 {
    let inv = 1.0 - t.clamp(0.0, 1.0);
    1.0 - inv.powi(5)
}
```

### 2.2 Sinusoidal Ease-In-Out (`ease_in_out`)
Used for continuous reversible disclosures (such as the operations console drawer) where symmetric acceleration and deceleration provide natural physical weight.

$$f(t) = \frac{1 - \cos(\pi t)}{2}$$

Implementation in `src/state/motion.rs`:
```rust
pub fn ease_in_out(t: f32) -> f32 {
    let c = t.clamp(0.0, 1.0);
    0.5 * (1.0 - (std::f32::consts::PI * c).cos())
}
```

---

## 3. Continuous Transitions: The `AnimatedScalar` Model

Continuous spatial animations (widths, heights) are governed by the `AnimatedScalar` state machine.

### 3.1 State Representation
```rust
pub struct AnimatedScalar {
    pub current: f32,
    pub start_value: f32,
    pub target_value: f32,
    pub duration: Duration,
    pub start_time: Option<Instant>,
    pub easing: fn(f32) -> f32,
}
```

### 3.2 Evaluation & Reversal Mechanics
1. **Sampling**: At any frame rendered at timestamp $T_{\text{now}}$:
   $$\Delta t = \text{saturating\_sub}(T_{\text{now}}, T_{\text{start}})$$
   $$p = \text{clamp}\left(\frac{\Delta t}{D}, 0.0, 1.0\right)$$
   $$v_{\text{current}} = v_{\text{start}} + (v_{\text{target}} - v_{\text{start}}) \cdot f(p)$$

2. **Mid-Flight Reversal (Retargeting)**:
   When a transition is interrupted before completion (e.g. user expands the sidebar while it is collapsing), `retarget()` samples the exact instantaneous $v_{\text{current}}$, sets $v_{\text{start}} = v_{\text{current}}$, and resets the timer with the new destination:
   ```rust
   pub fn retarget(&mut self, new_target: f32, duration: Duration, now: Instant, reduce_motion: bool) {
       if reduce_motion {
           self.snap(new_target);
           return;
       }
       self.start_value = self.current;
       self.target_value = new_target;
       self.duration = duration;
       self.start_time = Some(now);
   }
   ```
   This guarantees zero positional discontinuities or visual snapping.

---

## 4. Discrete Transitions: GPUI `with_animation`

Discrete transitions occur when the DOM structure changes (e.g., navigating to Settings, or switching between Overview and Dependencies tabs).

### 4.1 Invariant: Children Must Precede Animation
In GPUI 0.2.2, `AnimationElement<Self>` implements `IntoElement`, but does not expose `.child(...)`. Therefore, child elements must be attached to the `div()` *prior* to wrapping with `.with_animation()`:

```rust
div()
    .id("tab_content")
    .size_full()
    .child(tab_body)
    .with_animation(
        ("tab-fade", tab_epoch as usize),
        Animation::new(MotionDurations::FAST).with_easing(gpui::ease_out_quint()),
        |element, delta| element.opacity(delta),
    )
```

### 4.2 Stationary Hero Header Contract
When switching inspector tabs, the top hero section (`InspectorHeader`: package title, version, status pills, action buttons, and tab switcher) remains stationary outside the animated container. Only the tab content container participates in the 120ms cross-fade.

---

## 5. Reduced Motion Policy (`MotionPolicy`)

When `reduce_motion = true`:
1. `AnimatedScalar::retarget()` immediately invokes `.snap(new_target)`, establishing target geometry in 0 frames.
2. Discrete transitions bypass `.with_animation(...)` entirely, rendering direct opacity 1.0 elements.
3. Toasts skip fade-in and decay animations, rendering at full opacity until dismissed.
