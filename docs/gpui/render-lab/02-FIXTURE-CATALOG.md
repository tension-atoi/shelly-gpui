# Render Lab Fixture Catalog (RENDER-00 Canonical Corpus)

## 0. Corpus Provenance

The Render Lab catalog contains exactly **46 canonical fixtures** derived directly from two immutable source board artifacts:

| Source Artifact | Dimensions | SHA-256 Digest |
|---|---|---|
| `gnosix-board-GHIJ-source.png` | 2048 × 2048 | `96d94a6b1cd7c5680c5e1432d49acbb92be0873d60eff34408c9ffbb5675ac16` |
| `gnosix-board-K-source.png` | 2048 × 2048 | `950682c3fa7ef9e51d1b8604c1f15d25d5fe1e446b2b47c9cc9120387b8726b8` |

---

## 1. Complete Fixture Registry

> **Note on Capability Classification**: In RENDER-00, every visual fixture's canonical status is **`UNKNOWN`**. The "Initial Hypothesis" column below represents a non-canonical research conjecture only; actual verdicts are earned through evidence in RENDER-01 through RENDER-03.

### Group G — Fields (Board G)
| Fixture ID | Cell | Source Label | Seed | Initial Hypothesis | Canonical Capability |
|---|---|---|---|---|---|
| `field.zero-positive-scalar` | 1 | Zero-to-positive scalar | 1001 | Native / Linear gradient | `UNKNOWN` |
| `field.signed-voltage` | 2 | Signed voltage | 1002 | Composable (bipolar split) | `UNKNOWN` |
| `field.current-magnitude` | 3 | Current magnitude | 1003 | Native / Composable | `UNKNOWN` |
| `field.current-direction` | 4 | Current direction | 1004 | Composable vector field | `UNKNOWN` |
| `field.overload-heat` | 5 | Overload heat | 1005 | Composable heat map | `UNKNOWN` |
| `field.diagnostic-confidence` | 6 | Diagnostic confidence | 1006 | Native 2-stop gradient | `UNKNOWN` |
| `field.component-stress` | 7 | Component stress | 1007 | Composable multi-stop | `UNKNOWN` |
| `field.selection-density` | 8 | Selection density | 1008 | Composable density plane | `UNKNOWN` |

### Group H — Depth (Board H)
| Fixture ID | Cell | Source Label | Seed | Initial Hypothesis | Canonical Capability |
|---|---|---|---|---|---|
| `depth.contact-shadow` | 1 | Contact shadow | 2001 | Native box-shadow | `UNKNOWN` |
| `depth.component-lift-shadow` | 2 | Component lift shadow | 2002 | Native diffused shadow | `UNKNOWN` |
| `depth.recessed-socket-shadow` | 3 | Recessed socket shadow | 2003 | Composable inner shadow | `UNKNOWN` |
| `depth.inset-panel-inner-shadow` | 4 | Inset panel border with inner shadow | 2004 | Composable border + shadow | `UNKNOWN` |
| `depth.raised-instrument-subtle-bevel` | 5 | Raised instrument body subtle bevel | 2005 | Composable multi-border | `UNKNOWN` |
| `depth.soft-edge-top-highlight` | 6 | Soft edge highlight top line | 2006 | Native border top | `UNKNOWN` |
| `depth.rim-highlight-outline` | 7 | Rim highlight outline | 2007 | Native continuous outline | `UNKNOWN` |
| `depth.shallow-bevel-3d-edge` | 8 | Shallow bevel 3D edge | 2008 | Composable dual-tone bevel | `UNKNOWN` |

### Group I — Optical (Board I)
| Fixture ID | Cell | Source Label | Seed | Initial Hypothesis | Canonical Capability |
|---|---|---|---|---|---|
| `optical.radial-fade` | 1 | Radial fade | 3001 | Composable / TextureProof | `UNKNOWN` |
| `optical.directional-fade` | 2 | Directional fade | 3002 | Native linear opacity | `UNKNOWN` |
| `optical.soft-rectangle-rounded` | 3 | Soft rectangle rounded | 3003 | Native box-shadow glow | `UNKNOWN` |
| `optical.soft-circle-radial` | 4 | Soft circle radial | 3004 | Composable radial falloff | `UNKNOWN` |
| `optical.edge-vignette` | 5 | Edge vignette corners darker | 3005 | Composable / TextureProof | `UNKNOWN` |
| `optical.local-focus` | 6 | Local focus center bright | 3006 | Composable center bloom | `UNKNOWN` |
| `optical.energized-wire-glow` | 7 | Energized wire yellow glow | 3007 | Composable core + bloom | `UNKNOWN` |
| `optical.heat-region-glow` | 8 | Heat region orange glow | 3008 | Composable thermal glow | `UNKNOWN` |

### Group J — Materials (Board J)
| Fixture ID | Cell | Source Label | Seed | Initial Hypothesis | Canonical Capability |
|---|---|---|---|---|---|
| `material.painted-metal.matte-anthracite` | 1 | Matte anthracite | 4001 | Native solid / subtle tint | `UNKNOWN` |
| `material.painted-metal.signal-orange` | 2 | Signal orange painted metal | 4002 | Native solid / highlight | `UNKNOWN` |
| `material.painted-metal.beret-green` | 3 | Beret green painted metal | 4003 | Native solid / highlight | `UNKNOWN` |
| `material.painted-metal.royal-blue` | 4 | Royal blue painted metal | 4004 | Native solid / highlight | `UNKNOWN` |
| `material.brushed-aluminum` | 5 | Brushed aluminum | 4005 | TextureProof (anisotropic) | `UNKNOWN` |
| `material.dark-anodized-aluminum` | 6 | Dark anodized aluminum | 4006 | Composable satin sheen | `UNKNOWN` |
| `material.warm-paper` | 7 | Warm paper | 4007 | TextureProof (fibrous grain) | `UNKNOWN` |
| `material.smoked-plastic` | 8 | Smoked plastic | 4008 | Composable translucent tint | `UNKNOWN` |

### Group K — Microstructure (Board K)
| Fixture ID | Cell | Source Label | Seed | Initial Hypothesis | Canonical Capability |
|---|---|---|---|---|---|
| `micro.k01` | 1 | None (unlabeled) | 5001 | TextureProof (uniform noise) | `UNKNOWN` |
| `micro.k02` | 2 | None (unlabeled) | 5002 | TextureProof (blue noise) | `UNKNOWN` |
| `micro.k03` | 3 | None (unlabeled) | 5003 | TextureProof (fine grit) | `UNKNOWN` |
| `micro.k04` | 4 | None (unlabeled) | 5004 | TextureProof (brushed micro) | `UNKNOWN` |
| `micro.k05` | 5 | None (unlabeled) | 5005 | TextureProof (anisotropic grain) | `UNKNOWN` |
| `micro.k06` | 6 | None (unlabeled) | 5006 | TextureProof (cellular pattern) | `UNKNOWN` |
| `micro.k07` | 7 | None (unlabeled) | 5007 | TextureProof (stochastic stipple) | `UNKNOWN` |
| `micro.k08` | 8 | None (unlabeled) | 5008 | TextureProof (perlin lattice) | `UNKNOWN` |
| `micro.k09` | 9 | None (unlabeled) | 5009 | TextureProof (halftone mesh) | `UNKNOWN` |
| `micro.k10` | 10 | None (unlabeled) | 5010 | TextureProof (woven matrix) | `UNKNOWN` |
| `micro.k11` | 11 | None (unlabeled) | 5011 | TextureProof (sandblast relief) | `UNKNOWN` |
| `micro.k12` | 12 | None (unlabeled) | 5012 | TextureProof (etched fiber) | `UNKNOWN` |
| `micro.k13` | 13 | None (unlabeled) | 5013 | TextureProof (fine dither) | `UNKNOWN` |
| `micro.k14` | 14 | None (unlabeled) | 5014 | TextureProof (coarse grain) | `UNKNOWN` |
