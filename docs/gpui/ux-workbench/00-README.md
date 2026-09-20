# UX Workbench Redesign — Phase UX-01: Shell Architecture, Strict Inspector 3/7 Ratio & Adaptive Layout

## Context & Objectives
Phase UX-01 executes the foundational architecture of the **Shelly GPUI UX Workbench Redesign Contract**. The goal of this phase is to establish mathematical guarantees and ergonomic physical boundaries for the primary workstation canvas before any visual styling overhauls:

1. **Strict 3/7 Inspector Cap**:
   $$w_{\text{inspector}} \le \min\left(520\text{px},\, w_{\text{content}} \times \frac{3}{7}\right)$$
   with a hard floor of $w_{\text{inspector}} \ge 320\text{px}$ (`INSPECTOR_MIN_USABLE`).
2. **Zero Overlay / Zero Occlusion**:
   The inspector is physically docked alongside or stacked below the results list. It never floats, overlays, or occludes the list or interactive controls.
3. **Adaptive Layout Modes**:
   - `Horizontal` split mode when $w_{\text{content}} \ge 746.67\text{px}$.
   - `Stacked` detail fallback mode when $w_{\text{content}} < 746.67\text{px}$ (docked below the list).
4. **Sticky Query Surface**:
   The `QueryWorkbench` remains permanently anchored at the top of the Results pane; results scroll cleanly below it via `uniform_list`.
5. **Rigorous Pure Geometry**:
   Pure mathematical bounds derivation (`compute_splitter_bounds`), delta-isolated resize math (`compute_splitter_width`), and unit tests covering all reference viewports.

---

## Documents in this Phase
- [`00-README.md`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/00-README.md): Overview and roadmap of Phase UX-01.
- [`01-PRODUCT-PRINCIPLES.md`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/01-PRODUCT-PRINCIPLES.md): Core tenets: Native workstation vs web app, ergonomics, beauty, and strict hierarchy.
- [`02-LAYOUT-AND-INSPECTOR.md`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/02-LAYOUT-AND-INSPECTOR.md): Mathematical derivation of splitter limits, ratio invariants, and adaptive layout modes.
- [`08-ACCEPTANCE-MATRIX.md`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/08-ACCEPTANCE-MATRIX.md): Verification matrix for all Phase UX-01 requirements.
- [`09-RUNTIME-EVIDENCE.md`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/09-RUNTIME-EVIDENCE.md): Live Wayland execution evidence, Hyprland metrics, and package verification.
