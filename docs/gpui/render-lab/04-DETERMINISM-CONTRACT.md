# Determinism Contract & State Model

## 0. Determinism Invariant

In the Render Lab, visual state must be strictly reproducible:

```text
Given identical:
    FixtureId
    FrozenTime (t)
    Seed
    TopologyVariant
    MotionVariant
    QualityLevel

the resulting output projection MUST be deterministic.
```

---

## 1. Clock Model: Realtime vs. Frozen Time

The Render Lab supports two clock modes:

```rust
pub enum ClockMode {
    Realtime,
    Frozen(f32),
}
```

1. **`Realtime`**: Free-running wall clock. Used for interactive motion observation.
2. **`Frozen(t)`**: Explicitly locked time parameter in seconds (e.g. `t = 0.500s`).
   - Essential for golden pixel comparisons, screenshot proofs, and test automation.
   - Zero wall-clock dependence.
   - Disables all non-deterministic timers or sampling.

CLI Command:
```bash
shelly-gpui render-lab time 0.500
```

---

## 2. Lab-Assigned Deterministic Seeds

All procedural patterns (especially microstructures in Group K) require explicit pseudorandom or quasi-random seeds:

- **Group G Seeds**: `1001` through `1008`
- **Group H Seeds**: `2001` through `2008`
- **Group I Seeds**: `3001` through `3008`
- **Group J Seeds**: `4001` through `4008`
- **Group K Seeds**: `5001` through `5014`

> **Provenance Note**: These seeds are **lab-assigned deterministic parameters** to ensure test repeatability; they are not historical attributes of the original visual source drawings.
