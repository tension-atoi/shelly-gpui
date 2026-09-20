# Shelly GPUI — Query Workbench, Filtering & Sorting

## 1. Responsive 2-Row Query Workbench

The Query Workbench replaces the crowded single-row filter bar with an ergonomic two-tier layout:

```text
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ 🔍 [Search packages and apps...                                                 ] [×]  │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ [All] [ALPM] [AUR] [Flatpak] [AppImage]  Sort: Relevance ▾  State: All ▾  14 pkgs [▦][≡]│
└────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Row 1 (`SEARCH_INPUT_HEIGHT = 40.0px`)**: Unobstructed native search input dominating the list pane width.
- **Row 2 (`QUERY_CONTROLS_HEIGHT = 34.0px`)**: Query manipulation controls, distribution source pills, sorting selector, state filter, result count/progress indicator, and view mode switcher.

---

## 2. Adaptive Breakpoints

The workbench measures the list pane's usable width dynamically:

| Breakpoint | Width Threshold | Layout Adaptation |
|---|---|---|
| **Wide** | $\ge 540\text{px}$ | Full source pills (`All`, `ALPM`, `AUR`, `Flatpak`, `AppImage`), expanded active filter summary pill (`Reset: Official · Installed · Name (A–Z)`), sort button, state filter button, view mode switcher. |
| **Medium** | $380\text{px} \le w < 540\text{px}$ | Compact source pill labels (`All`, `ALPM`, `AUR`, `FP`, `AI`), compact Reset button, sort button, state filter button, view mode switcher. |
| **Narrow** | $< 380\text{px}$ | Minimal cycling source button (`Src: ALPM`), cycling sort button, view mode switcher. Zero truncation or text clipping. |

---

## 3. Client-Side Sorting Engine (`SortMode`)

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
To prevent list jitter during navigation or filtering, all sort modes implement deterministic secondary and tertiary tie-breakers:
1. `NameAsc`: `name.to_lowercase()` $\to$ `name` $\to$ `source_type`
2. `NameDesc`: `reverse(name.to_lowercase())` $\to$ `name` $\to$ `source_type`
3. `Source`: distribution rank (`Standard/ALPM = 0`, `AUR = 1`, `Flatpak = 2`, `AppImage = 3`) $\to$ `name.to_lowercase()` $\to$ `name`
4. `InstalledFirst`: `is_installed (true > false)` $\to$ `name.to_lowercase()` $\to$ `name` $\to$ `source_type`
5. `UpdatesFirst`: `has_update (true > false)` $\to$ `name.to_lowercase()` $\to$ `name` $\to$ `source_type`

---

## 4. Package State Filter (`PackageStateFilter`)

A lightweight client-side state predicate:
- `All`: Passes all packages.
- `Installed`: Passes packages where `is_installed == true`.
- `NotInstalled`: Passes packages where `is_installed == false`.
- `UpdatesAvailable`: Passes packages where `has_update == true`.

---

## 5. Dual Usable Geometry Invariant

The workstation layout guarantees that neither pane can squeeze the other into an unusable state:
- `LIST_MIN_USABLE = 340.0px`
- `INSPECTOR_MIN_USABLE = 320.0px`
- `SPLITTER_WIDTH = 5.0px`

```rust
pub fn compute_splitter_width(
    start_width: f32,
    start_pointer_x: f32,
    current_pointer_x: f32,
    total_viewport_width: f32,
) -> f32 {
    let delta = current_pointer_x - start_pointer_x;
    let proposed = start_width + delta;

    let max_allowed = (total_viewport_width
        - UiMetrics::SIDEBAR_EXPANDED
        - UiMetrics::SPLITTER_WIDTH
        - UiMetrics::INSPECTOR_MIN_USABLE)
        .max(UiMetrics::LIST_MIN_USABLE);

    let effective_max = max_allowed.min(700.0);
    proposed.clamp(UiMetrics::LIST_MIN_USABLE, effective_max)
}
```
Even on small laptop screens (e.g. $1024 \times 680$), the inspector is guaranteed at least $320\text{px}$ of usable horizontal space.
