# 09 — Runtime Evidence: Phase UX-01 Live Wayland Execution

## 1. Environment & Package Verification
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

## 2. Process & Binary Integrity
- **Running Binary Path**: `/usr/lib/shelly/shelly-gpui-bin`
- **Desktop Wrapper**: `/usr/bin/shelly-gpui`
- **Process ID (PID)**: `2733610`
- **Proc Symlink Verification**:
  ```text
  /proc/2733610/exe -> /usr/lib/shelly/shelly-gpui-bin
  ```

---

## 3. Hyprland Client Geometry
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

## 4. Visual Evidence
- **Screenshot Path**: [`docs/gpui/ux-workbench/evidence_ux01_shell_geometry.png`](file:///home/tension_atoi/Projects/shelly-gpui/docs/gpui/ux-workbench/evidence_ux01_shell_geometry.png)
- **Capture Method**:
  ```sh
  WAYLAND_DISPLAY=wayland-1 XDG_RUNTIME_DIR=/run/user/1000 grim -g "658,653 1263x1302" docs/gpui/ux-workbench/evidence_ux01_shell_geometry.png
  ```

### Visual Verification Checklist
1. **Expanded Sidebar (190px)**: Left pane displays branded logo, title, and all destination buttons (`Browse` active).
2. **Sticky Query Surface**: `SearchInputView` spans full list width, with filter dropdowns and source pills anchored immediately below.
3. **Isolated Draggable Splitter**: 5px vertical bar between Results list and Inspector.
4. **Strict 3/7 Inspector Cap**: Inspector pane is docked on the right, strictly capped at $\le \min(520\text{px}, w_{\text{content}} \times \frac{3}{7})$, with zero overlay over results.
5. **Docked Console Drawer**: Anchored at the bottom without floating or overlapping primary canvas.
