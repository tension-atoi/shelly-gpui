# 02 — Layout Architecture & Inspector Mathematics

## 1. Geometry and Splitter Clamp Contract
The workstation content canvas receives width $w_{\text{content}} = w_{\text{window}} - w_{\text{sidebar}}$, where $w_{\text{sidebar}}$ is either $56\text{px}$ (collapsed) or $190\text{px}$ (expanded).

The horizontal layout divides $w_{\text{content}}$ into:
$$w_{\text{content}} = L + S + I$$
where:
- $L$ is the Results list pane width,
- $S = 5\text{px}$ is the Splitter width (`UiMetrics::SPLITTER_WIDTH`),
- $I$ is the Inspector pane width.

### Invariant Constraints
1. **Inspector Maximum Width & Ratio**:
   $$I \le I_{\max} = \min\left(520.0,\, w_{\text{content}} \times \frac{3}{7}\right)$$
2. **Inspector Usable Floor**:
   $$I \ge I_{\min} = 320.0 \quad (\text{UiMetrics::INSPECTOR\_MIN\_USABLE})$$
3. **List Usable Floor**:
   $$L \ge L_{\min\_usable} = 340.0 \quad (\text{UiMetrics::LIST\_MIN\_USABLE})$$

### Bounds Derivation for $L$
From $I = w_{\text{content}} - S - L$:
- $I \le I_{\max} \iff w_{\text{content}} - S - L \le I_{\max} \iff L \ge w_{\text{content}} - S - I_{\max}$
  Therefore:
  $$L_{\min} = \max\left(340.0,\, w_{\text{content}} - S - I_{\max}\right)$$
- $I \ge I_{\min} \iff w_{\text{content}} - S - L \ge I_{\min} \iff L \le w_{\text{content}} - S - I_{\min}$
  Therefore:
  $$L_{\max} = \max\left(L_{\min},\, w_{\text{content}} - S - I_{\min}\right)$$

Clamping any requested list width $L \in [L_{\min}, L_{\max}]$ mathematically guarantees that $I \in [I_{\min}, I_{\max}]$ and $L \ge 340.0$.

---

## 2. Adaptive Layout Threshold
For $I_{\min} \le I_{\max}$ to hold:
$$320.0 \le w_{\text{content}} \times \frac{3}{7} \iff w_{\text{content}} \ge 320.0 \times \frac{7}{3} = \frac{2240}{3} \approx 746.6667\text{px}$$

`UiMetrics::HORIZONTAL_SPLIT_MIN_CONTENT_WIDTH = 320.0 * (7.0 / 3.0)` (~746.67px).

- **$w_{\text{content}} \ge 746.67\text{px}$ $\implies$ `WorkstationLayoutMode::Horizontal`**:
  Results on left, draggable splitter in middle, inspector docked on right. Inspector strictly bounded to $\le \min(520\text{px}, \frac{3}{7} w_{\text{content}})$.
- **$w_{\text{content}} < 746.67\text{px}$ $\implies$ `WorkstationLayoutMode::Stacked`**:
  Results on top (`flex_1`), inspector docked below (`h(px(280.0))`). Zero overlay, zero occluding popups.

---

## 3. Splitter Resize Isolation
When the user drags the splitter, pointer positions are in window coordinates.
$$\Delta x = x_{\text{current}} - x_{\text{start}}$$
$$L_{\text{requested}} = L_{\text{start}} + \Delta x$$
Because $\Delta x$ is a pure differential between pointer positions:
$$(x_{\text{current}} + x_{\text{sidebar}}) - (x_{\text{start}} + x_{\text{sidebar}}) = x_{\text{current}} - x_{\text{start}} = \Delta x$$
Splitter resize behavior is mathematically invariant to whether the sidebar is collapsed or expanded.
