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
4. **`SHADER_REQUIRED`**: Cannot be reproduced with sufficient functional fidelity in the available primitives without performance cost or visual approximation incompatible with its contract. (Strictly forbidden in RENDER-01).
5. **`UNKNOWN`**: Catalogued baseline state. No verdict rendered until empirical confrontation.

---

## 1. The Promotion Ladder

Verdicts are strictly earned through evidence. Pre-classifying capabilities without test evidence is an engineering defect.

```text
RENDER-00
   └── All visual fixtures initialized to UNKNOWN.

RENDER-01 (Current Slice Closure)
   └── Empirical confrontation with stock GPUI 0.2.2 primitives across all 46 fixtures.
   └── Evidence-based promotion to NATIVE (12), COMPOSABLE (18), or TEXTURE_PROOF (16).
   └── 0 UNKNOWN remaining. 0 SHADER_REQUIRED (deferred to RENDER-03 gap measurement).

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

---

## 2. Candidate RENDER-01 Capability Observations (46/46)

| Fixture ID | Backend | Recipe | Verdict | Manifest Hash | Caveats |
|---|---|---|---|---|---|
| `field.zero-positive-scalar` | `gpui-stock-0.2.2` | `g01-linear-scalar-gradient/r1` | **`NATIVE`** | `eb5c1c8d5c81...` | Direct 1D linear scalar field natively supported by GPUI linear_gradient. 2D non-linear gradients require composition. |
| `field.signed-voltage` | `gpui-stock-0.2.2` | `g02-bipolar-split/r1` | **`COMPOSABLE`** | `2eaa3d665f27...` | Zero-crossing threshold composed from two adjacent opposite-polarity gradients. Sharp boundary or nonlinear transition requires quad tessellation or texture. |
| `field.current-magnitude` | `gpui-stock-0.2.2` | `g03-magnitude-contour-stops/r1` | **`COMPOSABLE`** | `70e69bacf9e0...` | Magnitude ramp composable via multi-stop linear gradient; discrete isoline steps achieved by stepped color stops. |
| `field.current-direction` | `gpui-stock-0.2.2` | `g04-streamline-vector-field/r1` | **`COMPOSABLE`** | `6486a1760ea6...` | Directional vector field expressed via composed vector paths; continuous dense vector direction field requires texture. |
| `field.overload-heat` | `gpui-stock-0.2.2` | `g05-thermal-multistop-gradient/r1` | **`COMPOSABLE`** | `c6d17989dc04...` | Thermal palette composed from layered linear gradients and centered radiant oval. |
| `field.diagnostic-confidence` | `gpui-stock-0.2.2` | `g06-probabilistic-confidence-band/r1` | **`NATIVE`** | `ec2e2374c4aa...` | 1D probability density cleanly maps to single stock linear_gradient. |
| `field.component-stress` | `gpui-stock-0.2.2` | `g07-stress-concentration-contour/r1` | **`COMPOSABLE`** | `eb9c1370938e...` | Stress concentration composed from concentric stress boundary shells. |
| `field.selection-density` | `gpui-stock-0.2.2` | `g08-continuous-density-plane/r1` | **`COMPOSABLE`** | `e75bdb57be48...` | 2D bilinear density plane approximated by cross-fading orthogonal linear gradients. |
| `depth.contact-shadow` | `gpui-stock-0.2.2` | `h01-single-box-shadow/r1` | **`NATIVE`** | `5069768d7b6c...` | Tight contact occlusion cleanly produced by stock BoxShadow with low blur radius. |
| `depth.component-lift-shadow` | `gpui-stock-0.2.2` | `h02-diffused-elevation-shadow/r1` | **`NATIVE`** | `d80e3f348b13...` | Elevation penumbra produced by stock BoxShadow with larger blur and vertical offset. |
| `depth.recessed-socket-shadow` | `gpui-stock-0.2.2` | `h03-cavity-occlusion-bevel/r1` | **`COMPOSABLE`** | `3babe69a08f4...` | GPUI has no native inner shadow; cavity depth is composed from directional 1px/2px inner bevel borders and linear cavity gradient. |
| `depth.inset-panel-inner-shadow` | `gpui-stock-0.2.2` | `h04-inset-panel-directional-bevel/r1` | **`COMPOSABLE`** | `29a5038fa2c2...` | Inner shadow illusion composed from directional border strokes and linear falloff child. |
| `depth.raised-instrument-subtle-bevel` | `gpui-stock-0.2.2` | `h05-dual-tone-linear-bevel/r1` | **`COMPOSABLE`** | `51cca0660a93...` | Physical bevel profile composed from opposing edge highlights, outer shadow, and subtle body gradient. |
| `depth.soft-edge-top-highlight` | `gpui-stock-0.2.2` | `h06-ambient-top-highlight-rim/r1` | **`NATIVE`** | `7292d8445074...` | Single top-edge specular line directly expressible via stock border_t_1 and border_color. |
| `depth.rim-highlight-outline` | `gpui-stock-0.2.2` | `h07-continuous-perimeter-rim/r1` | **`NATIVE`** | `9402939a065c...` | Uniform perimeter highlight directly expressible via stock border_1 and border_color. |
| `depth.shallow-bevel-3d-edge` | `gpui-stock-0.2.2` | `h08-prismatic-shallow-bevel/r1` | **`COMPOSABLE`** | `0dd8d52d7da3...` | Multi-stage prismatic bevel composed of concentric nested borders and contrasting elevation shadows. |
| `optical.radial-fade` | `gpui-stock-0.2.2` | `i01-concentric-falloff/r1` | **`COMPOSABLE`** | `ebeadcf55c3c...` | Stock GPUI lacks radial gradient primitive; radial fade is approximated by concentric geometric discs. High ring count has tessellation cost. |
| `optical.directional-fade` | `gpui-stock-0.2.2` | `i02-linear-directional-fade/r1` | **`NATIVE`** | `42349ccdd518...` | Linear optical fade directly supported by stock linear_gradient with alpha color stops. |
| `optical.soft-rectangle-rounded` | `gpui-stock-0.2.2` | `i03-soft-rounded-box-glow/r1` | **`NATIVE`** | `d2c45d34e639...` | Soft rounded glow boundary cleanly produced by stock BoxShadow with zero offset and positive spread. |
| `optical.soft-circle-radial` | `gpui-stock-0.2.2` | `i04-radial-disc-glow-boundary/r1` | **`COMPOSABLE`** | `c1c8d723c5e7...` | Two-scale glow halo composed from nested circular disc and multi-lobe BoxShadow. |
| `optical.edge-vignette` | `gpui-stock-0.2.2` | `i05-corner-darkening-vignette/r1` | **`COMPOSABLE`** | `0d3c444258e5...` | True radial vignette composed of peripheral linear gradients and corner shadow anchors. |
| `optical.local-focus` | `gpui-stock-0.2.2` | `i06-central-luminance-boost/r1` | **`COMPOSABLE`** | `6d0933186418...` | Focus spotlight composed of centered illuminated disc and ambient darkening mask. |
| `optical.energized-wire-glow` | `gpui-stock-0.2.2` | `i07-filament-core-bloom/r1` | **`COMPOSABLE`** | `d13ac542ef4a...` | Bloom illusion composed of crisp 2px central filament and dual-stage soft BoxShadow glow. |
| `optical.heat-region-glow` | `gpui-stock-0.2.2` | `i08-thermal-emission-envelope/r1` | **`COMPOSABLE`** | `e54427cc0964...` | Volumetric heat glow approximated by layered concentric thermal discs with nonlinear opacity falloff. |
| `material.painted-metal.matte-anthracite` | `gpui-stock-0.2.2` | `j01-matte-anthracite-coating/r1` | **`NATIVE`** | `bd979004d060...` | Diffuse matte industrial finish cleanly represented by stock solid fill with subtle structural edge highlight. |
| `material.painted-metal.signal-orange` | `gpui-stock-0.2.2` | `j02-signal-orange-coating/r1` | **`NATIVE`** | `ffe09cdce248...` | Vibrant painted metal finish cleanly represented by stock solid fill, 1px highlight border and subtle top sheen. |
| `material.painted-metal.beret-green` | `gpui-stock-0.2.2` | `j03-beret-green-coating/r1` | **`NATIVE`** | `d7b9145bfd6b...` | Muted tactical paint cleanly represented by stock solid fill, subtle gradient, and edge border. |
| `material.painted-metal.royal-blue` | `gpui-stock-0.2.2` | `j04-royal-blue-coating/r1` | **`NATIVE`** | `7720604f1410...` | Precision instrument enamel cleanly represented by stock solid fill, subtle gradient, and edge border. |
| `material.brushed-aluminum` | `gpui-stock-0.2.2` | `j05-brushed-sphere-texture/r1` | **`TEXTURE_PROOF`** | `ed697fda3308...` | Directional anisotropic grain and micro-streaks cannot be synthesized via vector primitives; proven through deterministic immutable memory texture. |
| `material.dark-anodized-aluminum` | `gpui-stock-0.2.2` | `j06-dark-anodized-satin/r1` | **`COMPOSABLE`** | `e83fbdc4a4f3...` | Satin anodized sheen composed of subtle multi-stage linear gradient, low-contrast specular rim, and dark metallic fill. |
| `material.warm-paper` | `gpui-stock-0.2.2` | `j07-warm-paper-fibrous-texture/r1` | **`TEXTURE_PROOF`** | `311da5160cf2...` | Paper fiber microstructure and stochastic pulp variation require texture synthesis; verified via immutable memory texture. |
| `material.smoked-plastic` | `gpui-stock-0.2.2` | `j08-smoked-polycarbonate-translucent/r1` | **`COMPOSABLE`** | `204a4e9a78d8...` | Optical transmission and surface reflection composed of translucent tinted fill, inner highlight line, and backdrop contrast. |
| `micro.k01` | `gpui-stock-0.2.2` | `k01-uniform-noise-texture/r1` | **`TEXTURE_PROOF`** | `715b027a95fb...` | Uniform high-frequency stochastic noise synthesized into deterministic immutable texture buffer. |
| `micro.k02` | `gpui-stock-0.2.2` | `k02-stratified-jitter-texture/r1` | **`TEXTURE_PROOF`** | `afd610f235e3...` | Stratified jitter grid marks synthesized into deterministic immutable texture buffer. |
| `micro.k03` | `gpui-stock-0.2.2` | `k03-fine-grit-texture/r1` | **`TEXTURE_PROOF`** | `a76c0102ec60...` | Fine multi-scale grit with stochastic particle points synthesized into deterministic immutable texture buffer. |
| `micro.k04` | `gpui-stock-0.2.2` | `k04-brushed-micro-texture/r1` | **`TEXTURE_PROOF`** | `44cdb776c604...` | Dense directional micro-scratch field synthesized into deterministic immutable texture buffer. |
| `micro.k05` | `gpui-stock-0.2.2` | `k05-anisotropic-grain-texture/r1` | **`TEXTURE_PROOF`** | `e87a46d19c10...` | Anisotropic directional noise field synthesized into deterministic immutable texture buffer. |
| `micro.k06` | `gpui-stock-0.2.2` | `k06-cellular-pattern-texture/r1` | **`TEXTURE_PROOF`** | `fb68081c7579...` | Cellular Voronoi distance field synthesized into deterministic immutable texture buffer. |
| `micro.k07` | `gpui-stock-0.2.2` | `k07-stochastic-stipple-texture/r1` | **`TEXTURE_PROOF`** | `f282622f8a86...` | Stochastic stipple point density synthesized into deterministic immutable texture buffer. |
| `micro.k08` | `gpui-stock-0.2.2` | `k08-value-noise-lattice-texture/r1` | **`TEXTURE_PROOF`** | `c2d2a9576b1e...` | Bilinear value noise lattice synthesized into deterministic immutable texture buffer. |
| `micro.k09` | `gpui-stock-0.2.2` | `k09-halftone-mesh-texture/r1` | **`TEXTURE_PROOF`** | `01925e672d4c...` | Geometric halftone dot mesh synthesized into deterministic immutable texture buffer. |
| `micro.k10` | `gpui-stock-0.2.2` | `k10-woven-matrix-texture/r1` | **`TEXTURE_PROOF`** | `352b67275609...` | Interleaved woven matrix threads synthesized into deterministic immutable texture buffer. |
| `micro.k11` | `gpui-stock-0.2.2` | `k11-crater-relief-texture/r1` | **`TEXTURE_PROOF`** | `996b5b17156e...` | Crater pore depressions with directional lighting synthesized into deterministic immutable texture buffer. |
| `micro.k12` | `gpui-stock-0.2.2` | `k12-etched-fiber-texture/r1` | **`TEXTURE_PROOF`** | `5faae4c199e8...` | Curved etched fiber segments synthesized into deterministic immutable texture buffer. |
| `micro.k13` | `gpui-stock-0.2.2` | `k13-fine-dither-texture/r1` | **`TEXTURE_PROOF`** | `8e92e5843956...` | Ordered Bayer matrix dither synthesized into deterministic immutable texture buffer. |
| `micro.k14` | `gpui-stock-0.2.2` | `k14-coarse-grain-texture/r1` | **`TEXTURE_PROOF`** | `f448eba7e86e...` | Coarse block grain clusters synthesized into deterministic immutable texture buffer. |

---

## 3. Summary Statistics

- **Total Canonical Fixtures**: 46 / 46 (100% evaluated)
- **NATIVE**: 12 (26.1%)
- **COMPOSABLE**: 18 (39.1%)
- **TEXTURE_PROOF**: 16 (34.8%)
- **UNKNOWN**: 0 (0.0%)
- **SHADER_REQUIRED**: 0 (Forbidden in RENDER-01; ceiling measured within stock primitives)
