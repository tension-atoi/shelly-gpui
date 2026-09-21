# Settings Authority & Atomic Configuration Specification

## Dual Authority Separation

Configuration in Shelly GPUI is strictly segregated across two distinct authorities:

| Authority | Disk Location | Scope |
|:---|:---|:---|
| **`gpui-ui`** | `~/.config/shelly/gpui-ui.json` | Presentation layout, themes, view mode, drawer dimensions, motion policy |
| **`settings`** | `~/.config/shelly/settings.json` | Backend source enablement (AUR, Flatpak, AppImage) and package manager flags |

## Supported Typed Settings

| Key | Default | Authority | Allowed Values / Validation | Description |
|:---|:---|:---|:---|:---|
| `theme` | `"dark"` | `gpui-ui` | `"dark"`, `"light"` | Application visual theme |
| `compact-view` | `false` | `gpui-ui` | `true`, `false` | Compact sidebar and card density |
| `reduce-motion` | `false` | `gpui-ui` | `true`, `false` | Snaps animations to zero-duration |
| `view-mode` | `"table"` | `gpui-ui` | `"table"`, `"cards"` | Workstation presentation surface |
| `log-drawer-open`| `false` | `gpui-ui` | `true`, `false` | Auto-open operation console drawer |
| `log-drawer-height` | `220.0` | `gpui-ui` | Float `120.0` .. `600.0` | Height in pixels of operation console |
| `aur-enabled` | `true` | `settings` | `true`, `false` | Arch User Repository backend |
| `flatpak-enabled` | `true` | `settings` | `true`, `false` | Flatpak application backend |
| `appimage-enabled`| `true` | `settings` | `true`, `false` | AppImage standalone backend |
| `cascade-delete` | `true` | `settings` | `true`, `false` | Remove unneeded dependencies |
| `remove-configs` | `true` | `settings` | `true`, `false` | Purge configuration files |
| `window-width` | `1280.0` | `gpui-ui` | Float >= `1024.0` | Restored window width |
| `window-height`| `840.0` | `gpui-ui` | Float >= `680.0` | Restored window height |

## Atomic Filesystem Guarantees

In accordance with strict system reliability standards, configuration files are never written in place or truncated directly.

### Atomic Write Algorithm:
1. **Sibling Temporary Path**: A unique temporary file is created in the exact same directory as the target configuration file:
   ```text
   .{filename}.tmp.{pid}.{nanoseconds}
   ```
   Writing in the same directory ensures that the file resides on the same filesystem/mount point, which is an absolute requirement for atomic POSIX rename (`rename(2)` / `renameat2(2)`).
2. **Payload Write**: Data is completely written to the temporary file.
3. **Data Sync**: `file.sync_all()` is executed to ensure all data and inode metadata are physically written to durable storage.
4. **Atomic Rename**: `std::fs::rename(&tmp_path, target)` atomically replaces the destination file. If power or the process is lost before this point, the destination file remains 100% intact.
5. **Parent Directory Sync**: The parent directory descriptor is opened and `sync_all()` is invoked to persist the directory entry update.
