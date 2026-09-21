# Render Lab Canon: Orthogonal Visual Axes & Composition Doctrine

## 0. Hard Architectural Invariant: Strict Axis Separation

Rendering in Shelly adheres to a strict separation of concerns across visual and behavioral dimensions:

```text
Topology
   ≠
Curvature
   ≠
Material
   ≠
Field
   ≠
Depth
   ≠
Optical Effect
   ≠
Motion
   ≠
Visual Style
```

Each dimension represents an independent, orthogonal authority:

1. **Topology**: The structural envelope and bounding semantics of a component (e.g., `floating-island`, `full-band`, `perimeter-hug`).
2. **Curvature**: Corner radius geometry, squircle/beveled contours, and continuous curvature math.
3. **Material**: The surface response, base substrate reflectance, diffuse absorption, and micro-roughness.
4. **Field**: Continuous scalar or vector spatial distributions (e.g., potential, temperature, stress, confidence).
5. **Depth**: Spatial elevation hierarchy, ambient occlusion, contact shadows, lift shadows, and bevel facets.
6. **Optical Effect**: Light emission, transmission, glow, bloom, blur, attenuation, and vignetting.
7. **Motion**: Behavioral timing, kinetic curves, continuity across interruptions, and Reduced Motion compliance.
8. **Visual Style**: An orchestrator that maps semantic surface roles to concrete recipes across the above authorities without conflating them.

---

## 1. The Doctrine of "Compose Before Paint"

Existing Gnosix principles mandate:

```text
semantic regions
      ↓
canonical geometry
      ↓
canonical material boundary
      ↓
visual style resolution
      ↓
material / depth / optical recipes
      ↓
renderer projection
```

### Prohibitions:
- A component must never declare hardcoded ad-hoc blur or paint parameters directly in its widget tree.
- Nested UI elements do **not** automatically spawn nested material surfaces ("glassmorphism component soup" is strictly forbidden).
- Visual depth or optical falloff must never mutate layout measurements or hit-test bounds. Input bounds are strictly CPU-semantic.
