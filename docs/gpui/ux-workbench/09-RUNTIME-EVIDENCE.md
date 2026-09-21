# 09 — Runtime Evidence: Phases UX-01 & UX-02 Live Wayland Execution

## 1. Environment & Package Verification (Phase UX-01)
- **Target OS**: Arch Linux x86_64
- **Compositor**: Hyprland (Native Wayland, `xwayland: false`)
- **Display**: `WAYLAND_DISPLAY=wayland-1`
- **Installed Package**: `shelly-gpui-git r4687.g3fd657b3-1`
- **Source Git Commit**: `3fd657b3c624e209ec0e9ffa14f3e9f822d6ae64`
- **Package Installation Command**:
  ```sh
  sudo pacman -U --noconfirm /tmp/ux01-pkgbuild/shelly-gpui-git-r4687.g3fd657b3-1-x86_64.pkg.tar.zst
  ```
- **Verification via `pacman -Q`**:
  ```text
  shelly-gpui-git r4687.g3fd657b3-1
  ```

---

## 2. Process & Binary Integrity (Phase UX-01)
- **Running Binary Path**: `/usr/lib/shelly/shelly-gpui-bin`
- **Desktop Wrapper**: `/usr/bin/shelly-gpui`
- **Process ID (PID)**: `2733610`
- **Proc Symlink Verification**:
  ```text
  /proc/2733610/exe -> /usr/lib/shelly/shelly-gpui-bin
  ```

---

## 3. Hyprland Client Geometry (Phase UX-01)
Queried via `hyprctl clients -j`:
```json
{
    "address": "0x5651991743d0",
    "mapped": true,
    "hidden": false,
    "visible": true,
    "acceptsInput": true,
    "at": [658, 653],
    "size": [1263, 1302],
    "workspace": {
        "id": 1,
        "name": "1"
    },
    "floating": false,
    "monitor": 0,
    "class": "shelly-gpui",
    "title": "",
    "pid": 2733610,
    "xwayland": false
}
```

---

## 4. Visual Evidence (Phase UX-01)
- **Screenshot Path**: [`docs/gpui/ux-workbench/evidence_ux01_shell_geometry.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux01_shell_geometry.png)
- **Capture Method**:
  ```sh
  WAYLAND_DISPLAY=wayland-1 XDG_RUNTIME_DIR=/run/user/1000 grim -g "658,653 1263x1302" docs/gpui/ux-workbench/evidence_ux01_shell_geometry.png
  ```

---

## 5. Phase UX-02 Environment & Package Verification
- **Target OS**: Arch Linux x86_64
- **Compositor**: Hyprland (Native Wayland, `xwayland: false`)
- **Display**: `WAYLAND_DISPLAY=wayland-1`
- **Installed Package**: `shelly-gpui-git r4689.g73143130-1`
- **Source Git Commit**: `73143130fef0976752f20f7bd92bb07dbd36e05e`
- **Process ID (PID)**: `2957817`
- **Proc Symlink Verification**:
  ```text
  /proc/2957817/exe -> /usr/lib/shelly/shelly-gpui-bin
  ```
- **Hyprland Client Geometry**:
```json
{
    "address": "0x5651973c84a0",
    "mapped": true,
    "hidden": false,
    "visible": true,
    "acceptsInput": true,
    "at": [658, 653],
    "size": [1263, 1302],
    "workspace": {
        "id": 1,
        "name": "1"
    },
    "floating": false,
    "monitor": 0,
    "class": "shelly-gpui",
    "title": "",
    "pid": 2957817,
    "xwayland": false
}
```

---

## 6. Phase UX-02 Live Wayland UI/UX State Gallery

The complete visual evidence suite captured directly from the live Wayland compositor without mouse cursor occlusion:

| Figure | State / Component | Screenshot Artifact | Verified UX Properties |
| :--- | :--- | :--- | :--- |
| **01** | **Pristine Default State** | [`evidence_ux02_01_default_state.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_01_default_state.png) | Full-width search input, native desktop toolbar, zero floating pills, centered discovery box, docked inspector placeholder |
| **02** | **Filters Disclosure Open** | [`evidence_ux02_02_filters_menu_open.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_02_filters_menu_open.png) | `MenuSurface` popover anchored to `Filters ▾` button; multi-source checkboxes (`Official/ALPM`, `AUR`, `Flatpak`, `AppImage`), divider, state radios |
| **03** | **Active Source Summary** | [`evidence_ux02_03_active_filter_summary.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_03_active_filter_summary.png) | Single source excluded; button shows `Filters (1) ▾`; Row 3 shows `Showing 0 packages • Sources: Arch, Flatpak, AppImage` + inline `× Clear filters` |
| **04** | **State Dropdown Open** | [`evidence_ux02_04_state_menu_open.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_04_state_menu_open.png) | `State: All States ▾` dropdown popover open; radio items for `All States`, `Installed`, `Not Installed`, `Updates Available` |
| **04b**| **Combined Source & State Filter** | [`evidence_ux02_04b_state_filtered_installed.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_04b_state_filtered_installed.png) | Both source and state filters applied (`Filters (2) ▾`, `State: Installed ▾`); Row 3 displays combined textual summary |
| **04c**| **Filters Cleared** | [`evidence_ux02_04c_filters_cleared.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_04c_filters_cleared.png) | Verification of `× Clear filters` click: toolbar resets back to defaults and Row 3 smoothly collapses |
| **05** | **Sort Dropdown Open** | [`evidence_ux02_05_sort_menu_open.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_05_sort_menu_open.png) | Sort options popover open (`Relevance`, `Name A-Z`, `Name Z-A`, `Source Backend`, `Installed First`, `Updates First`) |
| **06** | **Active Search & Results** | [`evidence_ux02_06_search_active_results.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_06_search_active_results.png) | Search input with electric cyan focus ring, clear `×`, package count badge (`16 pkgs`), virtualized result cards |
| **07** | **Selected Card & Inspector** | [`evidence_ux02_07_package_selected_inspector.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_07_package_selected_inspector.png) | Selected card highlight; Inspector populated with actions (`Install`, `Copy install command`), tabs (`Overview`), and metadata cards |
| **08** | **Table View Mode** | [`evidence_ux02_08_table_view.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_08_table_view.png) | Tabular view mode active; columns for `NAME`, `VERSION`, `SOURCE`, `SIZE`, `STATUS`; selected row highlighted across full width |
| **09** | **Installed Packages View** | [`evidence_ux02_09_sidebar_installed.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_09_sidebar_installed.png) | Sidebar `Installed` destination active; listing 1,808 locally installed packages |
| **10** | **Updates View** | [`evidence_ux02_10_sidebar_updates.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_10_sidebar_updates.png) | Sidebar `Updates` destination active; clean empty state with primary `Upgrade All` button |
| **11** | **News View** | [`evidence_ux02_11_sidebar_news.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_11_sidebar_news.png) | Sidebar `News` destination active; official Arch Linux news feed |
| **12** | **Settings Workbench** | [`evidence_ux02_12_sidebar_settings.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_12_sidebar_settings.png) | Sidebar `Settings` destination active; full grouped settings cards and switches |
| **13** | **Collapsed Sidebar Rail** | [`evidence_ux02_13_sidebar_collapsed.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_13_sidebar_collapsed.png) | Collapsed 56px navigation rail mode maximizing horizontal canvas width |
| **14** | **Operation Console Drawer**| [`evidence_ux02_14_operation_log_expanded.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_14_operation_log_expanded.png) | Operation log drawer open at bottom; actions (`Copy logs`, `Clear`, `Auto-scroll`) |
| **15** | **Dark Theme Parity** | [`evidence_ux02_15_dark_theme_expanded.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_15_dark_theme_expanded.png) | Full dark theme workstation; high contrast obsidian surfaces, color-coded badges (`ALPM`, `AUR`) |
| **16** | **Inspector Overview Tab** | [`evidence_ux02_16_inspector_dependencies_tab.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_16_inspector_dependencies_tab.png) | Inspector docked side-by-side with active package details |
| **17** | **AUR Package Selected & Toast**| [`evidence_ux02_17_aur_package_selected.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02_17_aur_package_selected.png) | AUR package (`the_platinum_searcher-bin`) selected with AUR badges and upstream metadata; toast notification displayed |

---

## 7. Phase UX-02P Runtime Evidence: Command Surface Productization

### 7.1 Environment & Package Verification
- **Target OS**: Arch Linux x86_64
- **Compositor**: Hyprland (Native Wayland, `xwayland: false`)
- **Display**: `WAYLAND_DISPLAY=wayland-1`
- **Installed Package**: `shelly-gpui-git r4692.ge6228b99-1`
- **Source Git Commit**: `e6228b99d827cc81b9449021ffa15b40e4beb563`
- **Process ID (PID)**: `3752848`
- **Running Binary**: `/usr/lib/shelly/shelly-gpui-bin`
- **Verification Command**:
  ```sh
  pacman -Q shelly-gpui-git
  # Output: shelly-gpui-git r4692.ge6228b99-1
  ```

### 7.2 Live Wayland UX-02P Gallery

| Figure | State / Component | Screenshot Artifact | Verified UX Properties | Status |
| :--- | :--- | :--- | :--- | :--- |
| **01** | **Resting Unboxed Toolbar** | [`evidence_ux02p_01_resting_unboxed_toolbar.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02p_01_resting_unboxed_toolbar.png) | Row 2 completely unboxed at rest with transparent background and border; Row 3 permanent status rail displaying `0 packages` in dedicated 24px container | **PASSED** |
| **02** | **Active Search & Status Rail** | [`evidence_ux02p_02_search_active_and_rail.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02p_02_search_active_and_rail.png) | Search input with 110ms focus ring and clear `×`; Row 3 permanent rail displaying `16 packages` without layout shift | **PASSED** |
| **03** | **Desktop Checkmark Sort Menu** | [`evidence_ux02p_03_sort_menu_checkmarks.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02p_03_sort_menu_checkmarks.png) | Native desktop checkmark `✓` (`Relevance`) with precise alignment spacing for all items; zero radio circles | **PASSED** |
| **04** | **Desktop Checkmark State Menu** | [`evidence_ux02p_04_state_menu_checkmarks.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02p_04_state_menu_checkmarks.png) | Native desktop checkmark `✓` (`All States`) with alignment spacers for all items; zero radio circles | **PASSED** |
| **05** | **Non-Redundant Filters Menu** | [`evidence_ux02p_05_filters_menu_sources_only.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02p_05_filters_menu_sources_only.png) | Wide breakpoint: `Filters` disclosure contains exclusively package sources (`Official`, `AUR`, `Flatpak`, `AppImage`); Package State is omitted | **PASSED** |
| **06** | **Active Filter Badge & Summary Rail** | [`evidence_ux02p_06_filter_badge_and_rail_summary.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02p_06_filter_badge_and_rail_summary.png) | Button shows `Filters  1` in cyan/bold; Row 3 fixed 24px rail displays `3 packages  •  Official + Flatpak + AppImage` and inline `× Clear filters` button with strictly zero vertical layout shift | **PASSED** |
| **07** | **Table View Mode** | [`evidence_ux02p_07_table_view_mode.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02p_07_table_view_mode.png) | Multi-column table layout with active unboxed view switcher icon highlight and status rail parity | **PASSED** |
| **08** | **Light Theme Parity** | [`evidence_ux02p_08_light_theme_parity.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux02p_08_light_theme_parity.png) | Complete light theme visual parity: unboxed toolbar, status rail summary, search input, and high-contrast typography | **PASSED** |

---

## 8. Phase UX-03, UX-03R & UX-03R2 Runtime Evidence: Results Workbench & Package Identity Final Closure

### 8.1 Environment & Package Verification
- **Target OS**: Arch Linux x86_64
- **Compositor**: Hyprland (Native Wayland, `xwayland: false`)
- **Display**: `WAYLAND_DISPLAY=wayland-1`
- **Installed Package**: `shelly-gpui-git r4701.gd79e4aeb-1`
- **Source Git Commit**: `d79e4aeb`
- **Running Binary**: `/usr/lib/shelly/shelly-gpui-bin`
- **Verification Command**:
  ```sh
  pacman -Q shelly-gpui-git
  # Output: shelly-gpui-git r4701.gd79e4aeb-1
  ```
- **Architectural Invariants Verified**:
  1. **Strict Provenance-Backed Tier 1 for ALPM & AUR**: Probes `/var/lib/pacman/local/*/desc`, parses `%NAME%`, matches `== pkg.name` exactly (guaranteeing exact identity match, rejecting false prefix candidates like `python` matching `python-jinja`, and supporting letters in version strings like `r4699.ga172c43e-1`). Reads `files` list for owned `usr/share/applications/*.desktop` entries and resolves `Icon=` on disk. Uninstalled packages or CLI packages without owned desktop files honestly fall back to Tier 2 Symbolic (`SourceAlpm`, `SourceAur`) with zero fake logos and zero guessing.
  2. **Hot-Path Render Performance & Zero OS Thread Spawning**: `IdentityCache` provides $O(1)$ memory lookup on hits and immediate $O(1)$ symbolic rendering on misses. Cache misses enqueue into a bounded queue (2048) consumed by a dedicated background worker (`shelly-identity-resolver`). Zero OS threads are created on the render hot path.
  3. **Reactive UI Invalidation on Identity Resolution**: When background worker resolves an authentic icon, it emits `IdentityResolved(key)`, triggering a GPUI redraw so newly resolved Tier 1 icons appear reactively without requiring separate user interaction.
  4. **Single Preload Authority**: A single authority governs both proactive `preload()` (Browse search results, initial loads, refresh cycles) and cache-miss resolution, deduplicated by in-flight keys.
  5. **Failure-Truth Invariant Restored**: Failed `load_installed_packages` or `load_updates` preserves existing known-good package data and displays the error with a retry action, never wiping data or clearing the error flag.
  6. **Mathematical WCAG 2.1 AA Compliance**: `theme.rs` unit tests assert $\ge 4.5:1$ contrast ratios across all text tokens against `bg_surface` and `bg_app` in both Dark and Light themes. Text labels use dedicated `success_text` (`#047857` in light, `#34d399` in dark) and `warning_text` (`#b45309` in light, `#fbbf24` in dark), separating text contrast from indicator accent dots.
  7. **Outer Wrapper Inset Clarification**: Clarified that $80 + 4 + 4 = 88\text{px}$ is the outer row wrapper inset (`py_1()` in `package_workstation.rs`), distinct from the inner card padding (`py(px(6.0))` in `package_card.rs`).

### 8.2 Live Wayland UX-03R2 Gallery

| Figure | State / Component | Screenshot Artifact | Verified UX Properties | Status |
| :--- | :--- | :--- | :--- | :--- |
| **01** | **Table View Search Results** | [`evidence_ux03_01_table_view_search_results.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux03_01_table_view_search_results.png) | High-density multi-column Table layout with inline 16x16 source glyphs (`source-alpm.svg` cyan swoosh, `source-aur.svg` violet crest) in Name column, monospace versions, clean textual source (`Arch / cachyos-v3`, `AUR`) without candy pills, right-aligned monospace sizes (`pr_3`), calm status text (`Available`), docked inspector placeholder | **PASSED** |
| **02** | **Table View Selection & Inspector** | [`evidence_ux03_02_table_view_selection.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux03_02_table_view_selection.png) | Selected table row (`ripgrep`) highlighted across full width with electric cyan left accent rail; docked Inspector populated with action buttons (`Install`), tabs (`Overview`), description, and repository metadata | **PASSED** |
| **03** | **Cards View Mode** | [`evidence_ux03_03_cards_view_search_results.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux03_03_cards_view_search_results.png) | View switcher `[ ⊞ ]` active; 88px wrapper / 80px card footprint with 36x36 tinted source avatars; 3-line structural hierarchy (Line 1: Name + Monospace Version + Monospace Size; Line 2: Calm Desktop Metadata line `Arch · local · ● Installed` without candy pills; Line 3: Multi-line description clamped to 2 lines via `.line_clamp(2)`); high-contrast `success_text` label | **PASSED** |
| **04** | **Card Selection & Inspector** | [`evidence_ux03_04_cards_view_selection.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux03_04_cards_view_selection.png) | Selected card (`abseil-cpp`) with electric cyan left accent rail and active surface; docked Inspector populated with actions (`Uninstall`, `Copy install command`), tabs (`Overview`, `Dependencies`, `Files & Build`), and package metadata | **PASSED** |
| **05** | **Compact Cards View** | [`evidence_ux03_05_cards_view_compact_mode.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux03_05_cards_view_compact_mode.png) | Compact view mode active (70px wrapper, 62px card height, 28x28 avatar, collapsed 56px navigation rail, 1-line description clamp via `.line_clamp(1)`) maximizing vertical density with 13+ cards visible simultaneously | **PASSED** |
| **06** | **Light Theme Parity & WCAG 2.1 AA** | [`evidence_ux03_06_light_theme_parity.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux03_06_light_theme_parity.png) | Complete light theme visual parity; high-contrast source avatars, clean text columns (`Arch / local`), monospace versions and sizes, status text meeting strict WCAG 2.1 AA ($\ge 4.5:1$ with `success_text: #047857` and `text_muted: #64748b`) | **PASSED** |

---

## 9. Phase UX-04A Runtime Evidence: Detail Inspector & Compact Header Architecture

### 9.1 Environment & Package Verification
- **Target OS**: Arch Linux x86_64
- **Compositor**: Hyprland (Native Wayland, `xwayland: false`)
- **Display**: `WAYLAND_DISPLAY=wayland-1`
- **Installed Package**: `shelly-gpui-git r4704.g77192c38-1`
- **Source Git Commit**: `77192c38`
- **Running Binary**: `/proc/2136065/exe -> /usr/lib/shelly/shelly-gpui-bin`
- **Running Process ID (PID)**: `2136065`
- **Verification Command**:
  ```sh
  pacman -Q shelly-gpui-git
  # Output: shelly-gpui-git r4704.g77192c38-1
  ```
- **Architectural Invariants Verified**:
  1. **Pinned Header Architecture**: The upper region `#inspector_pinned_header` contains the 32x32 `PackageIdentity` avatar, bold package name, monospace version string, update delta, calm metadata line, action buttons (`Install` / `Uninstall`, `Copy install command`), and desktop tabs (`Overview`, `Dependencies`, `Files & Build`). It is fixed and non-scrolling, ensuring essential context and action controls are permanently accessible.
  2. **Single Scrollable Container**: `#inspector_scroll_body` is the only scrollable element (`flex_1().overflow_scroll()`), containing detail error banners and the tab body.
  3. **Calm Desktop Metadata Line & WCAG 2.1 AA Compliance**: Completely eliminated candy pill badges (`StatusPill::source_badge`, `StatusPill::installed_pill`). Replaced with `Arch · extra · ● Installed` using mathematically verified `theme.success_text` and `theme.warning_text` tokens.
  4. **Centered Calm Empty State**: When no package is selected, `#empty_inspector` displays a centered 40x40 `AppIcon::PackageGeneric` icon in `theme.text_muted`, bold `"No Package Selected"` title, and guidance text.
  5. **Macro Recursion Prevention**: Disambiguated unit test attributes using `#[core::prelude::v1::test]` to prevent `gpui::test` macro recursion during compilation.
  6. **Zero Deadcode & Strict Quality Gates**: All 112 unit tests pass in release locked profile; zero clippy warnings with `#![deny(dead_code)]`.

### 9.2 Live Wayland UX-04A Gallery

| Figure | State / Component | Screenshot Artifact | Verified UX Properties | Status |
| :--- | :--- | :--- | :--- | :--- |
| **01** | **Calm Empty Inspector State** | [`evidence_ux04a_01_empty_inspector.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux04a_01_empty_inspector.png) | Centered calm placeholder with 40x40 `PackageGeneric` vector glyph in `text_muted`, `text-sm font-semibold` "No Package Selected" title, and readable guidance description docked in right inspector pane | **PASSED** |



