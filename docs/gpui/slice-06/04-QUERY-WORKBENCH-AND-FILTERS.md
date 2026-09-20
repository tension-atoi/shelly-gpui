# Shelly GPUI — Query Workbench, Filtering & Sorting (Slice-06R)

## 1. Responsive 3-Row Query Workbench

The Query Workbench in Slice-06R provides an ergonomic, full-width hierarchical query workstation:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Row 1: 🔍 [Search packages and apps...                                           ] [×]  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Row 2: Filters (2) ▾  [ALPM] [AUR]  State: All ▾  Sort: Relevance ▾  14 pkgs  [Cards|Table]│
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Row 3: [Official ×]  [AUR ×]  [State: Installed ×]  [Clear filters]                   │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Row 1 (`SEARCH_INPUT_HEIGHT = 38.0px`)**: Full-width native `SearchInput` dominating the list pane width (`w_full`, `flex_1`). The search icon, clear button `[×]`, and progress indicator are strictly `flex_none`, eliminating any collapse or clipping.
- **Row 2 (`QUERY_CONTROLS_HEIGHT = 32.0px`)**:
  - `Filters [badge] ▾`: Anchored popover containing Sources checkboxes (with live health status and inline Retry buttons) and Package State selection.
  - Quick Source Pills: Instant source toggle pills visible on Wide breakpoints ($\ge 640\text{px}$).
  - `State: <State> ▾`: Dropdown menu for selecting package state (`All States`, `Installed`, `Not Installed`, `Updates Available`).
  - `Sort: <Mode> ▾`: Dropdown menu for selecting deterministic client-side sorting.
  - Result count badge & View Mode switcher (`[Cards | Table]`).
- **Row 3 (Active Chips & Reset)**:
  - Removable chips for active non-default filters (`[Official ×]`, `[AUR ×]`, etc.).
  - `Clear filters` button that resets sources to all enabled and state filter to `All`, while strictly preserving the current search query and sort mode.

---

## 2. Real GPUI Menus (`MenuSurface`, `MenuCheckItem`, `MenuRadioItem`)

All controls use real anchored GPUI popover surfaces instead of cycling buttons:
- Anchored via `deferred(anchored().snap_to_window().child(MenuSurface::render(...)))`.
- Occluded layout prevents pointer events leaking to elements beneath the menu.
- Dismissal on click-outside via `.on_mouse_down_out()`.
- Keyboard accessibility: items are focusable with tab stops, highlighted with focus rings, and activatable via `Enter` or `Space`.

---

## 3. Adaptive Breakpoints

The workbench measures the list pane's usable width dynamically:

| Breakpoint | Width Threshold | Layout Adaptation |
|---|---|---|
| **Wide** | $\ge 640\text{px}$ | Full controls: `Filters [badge] ▾`, Quick source pills (`ALPM`, `AUR`, `Flatpak`, `AppImage`), `State ▾`, `Sort ▾`, Count badge, View mode switcher. |
| **Medium** | $460\text{px} \le w < 640\text{px}$ | Compact layout: Quick pills collapsed into `Filters ▾` popover, `State ▾`, `Sort ▾`, Count badge, View mode switcher. |
| **Narrow** | $< 460\text{px}$ | Minimal layout: `Filters ▾` popover (sources + state), `Sort ▾`, View mode switcher. Zero truncation or text clipping. |

---

## 4. Multi-Source Health & Failure Truth

- Each source's health is tracked individually via `SourceHealthMap`.
- If a source fails during a multi-source concurrent search, its partial results from successful sources are displayed, while the failed source displays `Failed` with an inline `Retry` button inside the Filters menu.
- Partial search results are never cached in `search_cache` to ensure complete accuracy on subsequent queries.
- Truthful error cards with `Retry` buttons are provided for:
  - Installed packages list failure.
  - Updates list failure.
  - News feed failure.
  - Package Details failure in Inspector.

---

## 5. Client-Side Sorting Engine (`SortMode`)

All sorting operations execute in memory on the currently loaded `Arc<[UnifiedPackage]>`, eliminating redundant backend queries and IPC latency.

```rust
pub enum SortMode {
    Relevance,
    NameAsc,
    NameDesc,
    Source,
    InstalledFirst,
    UpdatesFirst,
}
```

### Deterministic Tie-Breaking
1. `NameAsc`: `name.to_lowercase()` $\to$ `name` $\to$ `source_type`
2. `NameDesc`: `reverse(name.to_lowercase())` $\to$ `name` $\to$ `source_type`
3. `Source`: distribution rank (`Standard/ALPM = 0`, `AUR = 1`, `Flatpak = 2`, `AppImage = 3`) $\to$ `name.to_lowercase()` $\to$ `name`
4. `InstalledFirst`: `is_installed (true > false)` $\to$ `name.to_lowercase()` $\to$ `name` $\to$ `source_type`
5. `UpdatesFirst`: `has_update (true > false)` $\to$ `name.to_lowercase()` $\to$ `name` $\to$ `source_type`

---

## 6. Package State Filter (`PackageStateFilter`)

A lightweight client-side state predicate:
- `All`: Passes all packages.
- `Installed`: Passes packages where `is_installed == true`.
- `NotInstalled`: Passes packages where `is_installed == false`.
- `UpdatesAvailable`: Passes packages where `has_update == true`.

---

## 7. Dual Usable Geometry Invariant

The workstation layout guarantees that neither pane can squeeze the other into an unusable state:
- `LIST_MIN_USABLE = 340.0px`
- `INSPECTOR_MIN_USABLE = 320.0px`
- `SPLITTER_WIDTH = 5.0px`

Even on small laptop screens (e.g. $1024 \times 680$), the inspector is guaranteed at least $320\text{px}$ of usable horizontal space.
