# 03 — Phase UX-02: Query Workbench Architecture & Runtime UX Critique

## 1. Objectives & Executive Overview

Phase UX-02 executes the complete overhaul of the Shelly GPUI query and control surface, transforming it from a prototype-grade "chip/pill UI" into a professional native desktop workstation toolbar.

### Core Deliverables of Phase UX-02
1. **Eradication of Floating Pill/Chip UI**:
   - Total removal of `render_quick_pill` and floating capsule tags from the query area.
   - Elimination of visual clutter and fragmented clickable chips.
2. **Desktop Native Toolbar (Rows 1 & 2)**:
   - **Row 1**: Full-width dominant `SearchInputView` with tactile focus ring, clear `×` button, and search glyph.
   - **Row 2**: Native desktop toolbar grouping:
     - `Filters (N) ▾` disclosure button with non-default badge counter.
     - Dynamic package count badge (e.g., `16 pkgs`).
     - `State: {filter} ▾` dropdown button (All States, Installed, Not Installed, Updates Available).
     - `Sort: {mode} ▾` dropdown button (Relevance, Name A-Z, Name Z-A, Source Backend, Installed First, Updates First).
     - Segmented view-mode control (`[ Cards | Table ]`).
3. **Structured Disclosure Popovers (`MenuSurface`)**:
   - Popovers anchored directly beneath the trigger controls.
   - Distinct sections: Source selection with checkboxes (`Official/ALPM`, `AUR`, `Flatpak`, `AppImage`), section divider, and Package State selection with radio indicators.
   - Click-away dismissal and keyboard escape handling.
4. **Active Filter Textual Summary (Row 3)**:
   - When non-default filters are active, a single clean summary line appears:
     `Showing X packages • Sources: Arch, Flatpak • State: Installed`
   - Inline tactile `× Clear filters` button resetting all filters back to default with zero layout jank.
   - Smooth collapse when filters are in their pristine default state.

---

## 2. In-Depth UX Critique of the Compiled Runtime Build

A comprehensive visual and interactive audit was conducted across all 18 live Wayland runtime captures (`evidence_ux02_01` through `evidence_ux02_17`). Below is the structured critique of the current runtime build.

### A. The Query Surface (Search Bar & Toolbar)
- **Visual Presence & Dominance**:
  - The search input now properly spans the full width of the list pane, establishing an authoritative visual anchor.
  - The electric cyan focus ring (`#0284c7` / `#38bdf8`) gives immediate feedback when active.
  - The text cursor and trailing `×` clear icon operate reliably without obscuring input text.
- **Toolbar Balance & Rhythm**:
  - Row 2 maintains a clean, uniform 28px height across all buttons, dropdowns, and segmented toggles.
  - The `Filters (N) ▾` disclosure button visually communicates filter state through its numerical badge without adding visual noise.
  - The `[ Cards | Table ]` segmented control cleanly toggles layout modes without disturbing the sticky header.
- **Critique & Minor Observations**:
  - In Row 2, the `16 pkgs` badge sits between `Filters ▾` and `State ▾`. While informative during search queries, its placement can feel slightly crowded when all dropdown labels are long.
  - The empty discovery state previously had a lingering text reference ("Source pills filter results..."), which has been updated to reflect the new native filters menu.

### B. Popover Menus (`MenuSurface`)
- **Ergonomics & Layout**:
  - The `Filters` menu (`evidence_ux02_02_filters_menu_open.png`) groups distribution backends into clear checkbox rows with checkmark glyphs.
  - Dividers provide clear separation between source distribution backends and package state options.
  - The `State` (`evidence_ux02_04_state_menu_open.png`) and `Sort` (`evidence_ux02_05_sort_menu_open.png`) menus utilize radio indicators to indicate single-choice selection.
  - Shadows and subtle borders ensure popovers read as elevated surfaces above the list.

### C. Active Filter Summary & Reset Action (Row 3)
- **Feedback & Reversibility**:
  - In `evidence_ux02_03_active_filter_summary.png` and `evidence_ux02_04b_state_filtered_installed.png`, activating filters immediately exposes Row 3.
  - The textual summary accurately concatenates active sources (`Sources: Arch, Flatpak, AppImage`) and non-default states (`State: Installed`).
  - Clicking `× Clear filters` (`evidence_ux02_04c_filters_cleared.png`) immediately restores default scope and collapses Row 3 without flickering or viewport scroll dislocation.

### D. Results Pane: Cards vs. Table Mode
- **Cards Mode (`evidence_ux02_06`, `evidence_ux02_07`) — CRITICAL FINDING FOR PHASE UX-03**:
  - **Description Truncation / Text Clipping**: In the current `package_card.rs`, the fixed card layout results in vertical clipping of the package description text. In both light and dark themes, the bottom half of letters in the description line is cut off horizontally by the card container boundary.
  - **Visual Hierarchy**: Package names are bold and readable, but the badges (`ALPM`, `Available`) feel slightly generic and spaced close to the description.
  - **Conclusion**: The query workbench successfully feeds and controls the card list, but the cards themselves urgently require the dedicated overhaul planned for **Phase UX-03 (Cards)** to solve height calculation, typography baselines, badge styling, and action triggers.
- **Table Mode (`evidence_ux02_08`)**:
  - The tabular view provides high-density browsing with dedicated columns for `NAME`, `VERSION`, `SOURCE`, `SIZE`, and `STATUS`.
  - Row selection highlights the active package across the full column width.
  - The table provides a powerful, expert-oriented alternative to cards for users scanning thousands of packages.

### E. Inspector Pane Integration
- **Strict 3/7 Ratio Invariant**:
  - In all captures, the Inspector occupies exactly ~480px width (well under the 520px maximum cap and within the 3/7 content width ratio).
  - The Results list retains ample horizontal space (over 550px width), preventing any cramped feeling.
- **Content Organization**:
  - Header displays package name, version, backend badge (`ALPM`, `AUR`), state badge (`Available`, `Installed`), and repository tag (`cachyos-v3`, `aur`).
  - Primary `Install` and secondary `Copy install command` actions are prominently displayed.
  - Overview metadata cards are neatly arranged in a two-column grid.

### F. Shell Navigation & Theming
- **Sidebar Integration**:
  - Expanded mode (190px) provides clear navigation between `Browse`, `Installed`, `Updates`, `News`, and `Settings`.
  - Collapsed rail mode (56px) centers icons cleanly for maximum workspace width.
  - In `Installed` view (`evidence_ux02_09`), 1,808 packages are listed seamlessly.
- **Dark Theme (`evidence_ux02_15`)**:
  - Excellent contrast, deep obsidian background (`#0f131a`), refined borders, and vibrant accent highlights.
  - Color-coded source badges (`ALPM` cyan/blue, `AUR` purple) pop clearly against dark surfaces.

---

## 3. Transition to Phase UX-03

Phase UX-02 has successfully delivered a clean, professional, native query surface with zero pills and robust state controls. The runtime critique clearly isolates the remaining ergonomic deficiency to the **package cards themselves** (clipping, density, hierarchy), which is the exact and exclusive mandate of **Phase UX-03**.
