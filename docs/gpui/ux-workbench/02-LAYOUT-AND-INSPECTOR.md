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

---

## 4. Inspector Information Architecture & Compact Header Contract (Phase UX-04A)

### 4.1 Pinned Header vs. Independent Scroll Container
In previous iterations, the entire inspector container (`#inspector_scroll`) had `overflow_scroll()`, causing the package avatar, title, version, action buttons, and navigation tabs to scroll out of view when viewing lengthy package descriptions or dependencies.

Under Phase UX-04A, the Inspector enforces a strictly pinned desktop architecture:
1. **Root Inspector Frame** (`#inspector_container`): Non-scrolling vertical flex container taking full height of the inspector pane (`flex().flex_col().size_full().bg(theme.bg_app)`).
2. **Pinned Desktop Header** (`#inspector_pinned_header`): Fixed non-scrolling upper region containing:
   - **Identity & Title Row**: 32x32 `PackageIdentity` avatar, bold package name, monospace version string, update delta indicator, and calm metadata line.
   - **Action Bar**: Compact 24px action buttons (`Install` / `Uninstall`, `Copy install command`).
   - **Desktop Tab Bar**: Underline tabs (`Overview`, `Dependencies`, `Files & Build`) with active indicator line and full keyboard accessibility.
3. **Scrollable Body** (`#inspector_scroll_body`): The **ONLY** scrollable region within the inspector (`flex_1().overflow_scroll().px_5().py_4()`), hosting any runtime error banners and the animated tab content.

### 4.2 Compact Identity & Calm Metadata Line
- **No Giant Hero Card**: Eradicated `text-2xl` oversized hero typography and large rounded badges in favor of a balanced desktop header (`text-base font-bold`).
- **32x32 Package Identity Avatar**: Integrates `PackageIdentity::render_avatar(pkg, 32.0, 6.0, theme)` reusing the verified 3-tier provenance chain (Tier 1 authentic icon -> Tier 2 symbolic glyph -> Tier 3 generic fallback).
- **Calm Desktop Metadata Line**: Replaced all pill and chip visual vocabulary (`StatusPill::source_badge`, `StatusPill::installed_pill`) with a calm, readable metadata line:
  $$\text{Origin} \quad \cdot \quad \text{Repository} \quad \cdot \quad \bullet \quad \text{Status}$$
  Example: `Arch · extra · ● Installed` or `Flatpak · flathub · ● Available`.
- **WCAG 2.1 AA Compliant Text Tokens**: Status text uses `theme.success_text` ($\ge 9.1:1$ contrast) and `theme.warning_text` ($\ge 10.5:1$ contrast) rather than saturated accent dots, guaranteeing accessibility compliance across Dark and Light themes.

### 4.3 Centered Calm Empty State
When no package is selected from the Results Workbench table or cards, `#empty_inspector` presents a centered calm desktop placeholder:
- 40x40 vector glyph (`AppIcon::PackageGeneric`) in `theme.text_muted`,
- `text-sm font-semibold text_primary` title: `"No Package Selected"`,
- `text-xs text_muted` guidance description: `"Select a package from the workstation table to inspect its details, dependencies, and files."`

