# STYLE-00A Settings Authority

## 0. Persisted Key

`gpui-ui.json` gains one typed field:

```json
"visual_style": "standard"
```

- `#[serde(default)]` → `Standard`; legacy files without the field parse
  unchanged and render identically (default Standard + untouched `Theme`).
- Validation rejects anything outside `standard` / `transparency`.
- Atomic write path reused (`sibling tmp + fsync + rename + dir fsync`).
- `settings reset visual-style` restores `standard`; `reset all` covers it.

## 1. Draft Flow (Unchanged Shape)

`SettingsView` drafts `GpuiUiConfig` (now including `visual_style`):

```text
select in dropdown → draft mutated, is_dirty = true
Save               → atomic persist both files, is_dirty = false
Reset              → draft restored from committed state
CLI set while dirty → rejected with edit-conflict error (CONTROL-01 guard)
```

## 2. Dropdown Control

Section `APPEARANCE & DENSITY`, below Compact View Density:

- Trigger button shows the draft value (`Standard ▾` / `Transparency ▾`),
  focusable with visible focus ring, `Enter`/`Space`/click to open.
- Menu reuses the ratified `MenuSurface` + exclusive `MenuCheckmarkItem`
  pair, `deferred` + `anchored` below the trigger, `snap_to_window`.
- Roving highlight (`Up`/`Down`/`Home`/`End`), `Enter`/`Space` selects and
  closes, `Escape`/outside-click closes without selecting.
- Opening the menu never dirties the draft; only selection does.
- Static honesty note: Transparency projection arrives with STYLE-00B.

## 3. Live Application Path

```text
appearance style set transparency   (GUI running)
  → ControlCommand::SettingsSet { visual-style }
  → dirty-draft guard
  → ConfigManager::set_setting (atomic)
  → apply_committed_settings_to_runtime (gpui_config reloaded)
  → response only after canonical state reflects the change
appearance status  →  visual_style = transparency   (read-your-writes)
```

In STYLE-00A no view consumes `gpui_config.visual_style` for paint, so the
application step is state-only by design. STYLE-00B wires consumers.
