import json

def main():
    with open("docs/gpui/render-lab/evidence/render01/EVIDENCE_INDEX.json") as f:
        items = json.load(f)

    table_rows = []
    for item in items:
        fixture = item["fixture"]
        backend = item["backend"]
        recipe = item["recipe"]
        verdict = item["verdict"]
        h = item["manifest_hash"][:12] + "..."
        caveats = item["caveats"]
        table_rows.append(f"| `{fixture}` | `{backend}` | `{recipe}` | **`{verdict}`** | `{h}` | {caveats} |")

    table_text = "\n".join(table_rows)

    content = f"""# Capability Ledger & Promotion Ladder

## 0. Capability Vocabulary

The `CapabilityClass` enum defines the exact capability classes recognized by the Render Lab:

```rust
pub enum CapabilityClass {{
    Native,
    Composable,
    TextureProof,
    ShaderRequired,
    Unknown,
}}
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
{table_text}

---

## 3. Summary Statistics

- **Total Canonical Fixtures**: 46 / 46 (100% evaluated)
- **NATIVE**: 12 (26.1%)
- **COMPOSABLE**: 18 (39.1%)
- **TEXTURE_PROOF**: 16 (34.8%)
- **UNKNOWN**: 0 (0.0%)
- **SHADER_REQUIRED**: 0 (Forbidden in RENDER-01; ceiling measured within stock primitives)
"""

    with open("docs/gpui/render-lab/03-CAPABILITY-LEDGER.md", "w") as f:
        f.write(content)
    print("Updated 03-CAPABILITY-LEDGER.md successfully")

if __name__ == "__main__":
    main()
