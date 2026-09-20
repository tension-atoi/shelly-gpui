# Shelly GPUI — IPC Protocol & Privileged Operations

## 1. Privilege Elevation Under UI Mode (P0-A)

### Problem Statement
In previous slices, executing ALPM mutations from GPUI resulted in permission failures because `Shelly.Cli.Zig` required root access and only triggered elevation when attached to an interactive terminal (`is_terminal == true`). Under GUI execution, `is_terminal` was false, causing `install standard`, `remove standard`, and `upgrade all` to fail immediately.

### Resolution Architecture
In `Shelly.Cli.Zig`:
- Mutating commands inspect the `--ui-mode` flag.
- When running under non-root UID with `--ui-mode`, the CLI checks `runtime.elevation.isElevated()`. If not elevated, it spawns `pkexec` with the target sub-command and arguments, passing stdout/stderr through the pipes.
- Non-root user targets (e.g. `flatpak install --user`) bypass elevation.
- This invokes the standard system Polkit authentication dialog on the Wayland compositor, allowing the user to authorize the transaction seamlessly.

---

## 2. Structured Framing Protocol (P0-B)

`Shelly.Cli.Zig` communicates execution milestones via framed JSON strings in stdout:
```text
[JSON]<base64-encoded-utf8-json>[/JSON]
```

`Shelly.Ui.Gpui/src/backend/protocol.rs` provides `ProtocolDecoder`:
```rust
pub enum UiFrame {
    AlpmInfo(AlpmInfoPayload),
    AlpmError(AlpmErrorPayload),
    OperationProgress(OperationProgressPayload),
    TransactionEvent(TransactionEventPayload),
}
```

### Fault-Tolerant Decoding Rules
1. Non-framed lines pass through directly as raw log strings.
2. If `[JSON]` and `[/JSON]` markers are detected, the interior slice is extracted and decoded with `base64::engine::general_purpose::STANDARD`.
3. If JSON deserialization fails, the line is preserved as a diagnostic string with `[Malformed Frame]` prefix rather than discarding data or panicking.

---

## 3. Human Operation Console & Semantic Timeline (P0-C)

`ConsoleModel` (`Shelly.Ui.Gpui/src/state/console.rs`) manages two synchronized views:
1. **Raw Log Stream**: Line-buffered text stream of all process output.
2. **Semantic Timeline**: High-level status per distribution backend:
   - `Standard (ALPM)`: `Pending` $\to$ `InProgress` $\to$ `Success` / `Failed`
   - `AUR`: `Pending` $\to$ `InProgress` $\to$ `Success` / `Failed`
   - `Flatpak`: `Pending` $\to$ `InProgress` $\to$ `Success` / `Failed`
   - `AppImage`: `Pending` $\to$ `InProgress` $\to$ `Success` / `Failed`

When an operation fails (e.g. Polkit authorization denied or ALPM lock held), `ConsoleModel` automatically opens the console drawer and displays both the human-readable error summary and structured diagnostic details.
