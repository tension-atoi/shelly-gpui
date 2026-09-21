# STYLE-00B-R2 Evidence

## Status

Evidence is tracked canonically under this directory. The Standard A/B gate is
not closed: ResultSurface is exact, while NavigationRail and QueryChrome have
only invalid prior pairs and Menu was not captured.

## Transparency live convergence

Runtime: installed `shelly-gpui-git r4724.gb0fab6de-1`, PID `3133690`.

Command sequence:

```text
appearance style set transparency
appearance style get                         -> visual-style = transparency
appearance status                            -> Visual Style: transparency, Scheme: dark, Profile Rev: 1
appearance resolve navigation-rail           -> Ambient Chrome, Opaque: no, Content Scrim: yes
appearance resolve query-chrome               -> Interactive Chrome, Opaque: no, Content Scrim: yes
appearance resolve result-surface             -> Content Plane, Opaque: no, Content Scrim: yes
appearance resolve menu                       -> Elevated Surface, Opaque: no, Content Scrim: yes
settings get visual-style                    -> visual-style = transparency
appearance style set standard
appearance style get                         -> visual-style = standard
```

Transparency effects reported by `appearance status` and every role resolver:

```text
ContactDepth    NATIVE
RimResponse     NATIVE
Microstructure  TEXTURE_PROOF
BackdropBlur    UNKNOWN
```

## Geometry invariant

The same installed runtime was queried with `hyprctl clients -j` before,
during, and after the live switch:

```text
standard       at=[658,653] size=[1263,1302] mapped=true xwayland=false
transparency   at=[658,653] size=[1263,1302] mapped=true xwayland=false
standard       at=[658,653] size=[1263,1302] mapped=true xwayland=false
```

Result: `standard -> transparency -> standard` did not change geometry.

## Architecture provenance

Command:

```text
rg 'CapabilityClass|render_lab' Shelly.Ui.Gpui/src/visual_style Shelly.Ui.Gpui/src/render_lab
```

Output includes:

```text
Shelly.Ui.Gpui/src/visual_style/resolver.rs: use crate::render_lab::CapabilityClass;
Shelly.Ui.Gpui/src/render_lab/capability.rs: pub enum CapabilityClass {
```

Interpretation: `CapabilityClass` was not relocated. The resolver still imports
the shared classification from the Render Lab module. The accurate rule claim
is therefore "no new product paint dependency on Render Lab implementation":
the product paint path consumes `StyleProjection`; the resolver's capability
classification remains a shared architectural type dependency.

## Runtime provenance

```text
readlink /proc/3133690/exe
/usr/lib/shelly/shelly-gpui-bin

shelly-gpui --version
shelly-gpui 0.1.0 (control protocol v4)
```
