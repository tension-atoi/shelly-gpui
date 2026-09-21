# Capability Ledger & Promotion Ladder

## 0. Capability Vocabulary

The `CapabilityClass` enum defines the exact capability classes recognized by the Render Lab:

```rust
pub enum CapabilityClass {
    Native,
    Composable,
    TextureProof,
    ShaderRequired,
    Unknown,
}
```

### Definitions:

1. **`NATIVE`**: Directly reproducible at full design fidelity using stock GPUI 0.2.2 public primitives (e.g. solid fills, linear two-stop gradients, borders, corner radii, drop shadows).
2. **`COMPOSABLE`**: Reproducible by assembling multiple overlapping native primitives, clipping masks, or layout math without native engine changes.
3. **`TEXTURE_PROOF`**: Cannot be expressed via vector/box primitives, but can be pre-rendered or procedurally synthesized into an immutable memory texture/canvas buffer (e.g. static noise, anisotropic brush microtextures).
4. **`SHADER_REQUIRED`**: Cannot be reproduced with sufficient functional fidelity in the available primitives without performance cost or visual approximation incompatible with its contract.
5. **`UNKNOWN`**: Catalogued baseline state. No verdict rendered until empirical confrontation.

---

## 1. The Promotion Ladder

Verdicts are strictly earned through evidence. Pre-classifying capabilities without test evidence is an engineering defect.

```text
RENDER-00 (Current)
   └── All visual fixtures initialized to UNKNOWN.

RENDER-01
   └── Empirical confrontation with stock GPUI primitives.
   └── Evidence-based promotion to NATIVE, COMPOSABLE, or TEXTURE_PROOF.

RENDER-02 / RENDER-03
   └── Recipe stress-testing and composition ceiling discovery.
   └── Strict burden of proof for SHADER_REQUIRED:
       "SHADER_REQUIRED does not mean 'I would prefer this in a shader'.
        It means: the primitive cannot be reproduced with sufficient fidelity
        in available primitives without unacceptable performance degradation
        or visual approximation incompatible with its contract."

RENDER-04
   └── Renderer decision (only if SHADER_REQUIRED evidence warrants it).
```
