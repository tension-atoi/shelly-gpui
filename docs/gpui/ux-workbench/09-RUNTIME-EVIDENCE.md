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
