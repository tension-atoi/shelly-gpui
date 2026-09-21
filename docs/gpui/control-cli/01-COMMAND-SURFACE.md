# Control CLI — Command Surface Specification

The CLI surface supports global options and subcommands for window management, navigation, inspection, console logging, and configuration management.

## Global Options

- `-j`, `--json`: Emits machine-readable JSON for scripting, automated verification, and agent pipelines.
- `-h`, `--help`: Prints comprehensive help text detailing available commands and exits 0.
- `-V`, `--version`: Prints executable version and protocol version (`shelly-gpui 0.1.0 (control protocol v1)`).

## Runtime Commands

### `shelly-gpui open`
- **Behavior**:
  - If the GUI is currently running: brings the existing window to focus via IPC and Wayland/Hyprland focus dispatcher.
  - If the GUI is not running: launches the desktop GUI.
- **Output**:
  - Human: `Focused existing Shelly window`
  - JSON: `{"version": 1, "ok": true, "message": "Shelly window focused"}`

### `shelly-gpui focus`
- **Behavior**:
  - If the GUI is running: focuses the window.
  - If not running: reports error and exits with code 1.

### `shelly-gpui status [--json]`
- **Behavior**:
  - Reports current operational status of the frontend.
- **Fields Reported**:
  - `protocol_version`: `1`
  - `gui_running`: boolean (`true` or `false`)
  - `pid`: process identifier (`0` if offline)
  - `executable`: binary path
  - `destination`: active navigation view (`updates`, `installed`, `browse`, `news`, `settings`)
  - `query`: active search query string
  - `view_mode`: `table` or `cards`
  - `inspector_tab`: `overview`, `dependencies`, or `files`
  - `selected_package`: package name currently displayed in inspector (or `null`)
  - `operation_running`: boolean indicating active background task or search in flight

### `shelly-gpui quit`
- **Behavior**:
  - Gracefully terminates running GUI instance, cleans up Unix domain socket, and exits 0.
  - If already offline, reports offline and exits 1.

### `shelly-gpui navigate <destination>`
- **Arguments**: `updates`, `installed`, `browse`, `news`, `settings`.
- **Behavior**:
  - If online: navigates running view and triggers appropriate data loading.
  - If offline: boots GUI with startup intent applied to the specified destination.

### `shelly-gpui search <query>`
- **Behavior**:
  - Sets destination to Browse, enters query in the search input, debounces and dispatches search.
  - If offline: boots GUI with search startup intent.

### `shelly-gpui view <table|cards>`
- **Behavior**:
  - Switches workstation presentation mode between tabular rows and card tiles.
  - Persists preference in `gpui-ui.json`.
  - Works both online (live UI update + save) and offline (atomic save).

### `shelly-gpui inspect <package>`
- **Behavior**:
  - Finds package in active store results or constructs typed ALPM reference, selects it in `AppSession`, and loads metadata for the inspector pane.
  - If offline: boots GUI with inspect startup intent.

### `shelly-gpui inspector <tab>`
- **Arguments**: `overview`, `dependencies` (or `deps`), `files` (or `files-build`).
- **Behavior**:
  - Switches active tab in the package details inspector.

### `shelly-gpui logs <show|hide|clear>`
- **Behavior**:
  - `show`: opens bottom operation drawer.
  - `hide`: closes bottom operation drawer.
  - `clear`: clears in-memory log buffer and resets progress bar.
