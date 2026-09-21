# Control Protocol Specification (v1)

## Transport & Addressing

The control protocol runs over a Unix Domain Socket located at:
```text
$XDG_RUNTIME_DIR/shelly-gpui/control.sock
```
On typical Linux systems, this resolves to `/run/user/<UID>/shelly-gpui/control.sock`.

### File Security & Permissions
- The directory `$XDG_RUNTIME_DIR/shelly-gpui/` is created with mode `0700` (`rwx------`).
- The socket file `control.sock` is created with mode `0600` (`rw-------`).
- Only the current user session has read/write privileges.

## Framing & Message Format

Messages are framed using Newline-Delimited JSON (NDJSON): each request and response is a single UTF-8 encoded JSON object terminated by `\n` (`0x0A`).

```
Client                                                  Server (GPUI)
  |                                                          |
  | -------- ControlRequest { version: 1, ... } \n --------> |
  |                                                          |
  | <------- ControlResponse { version: 1, ... } \n <------- |
```

### Request Schema

```json
{
  "version": 1,
  "command": {
    "action": "navigate",
    "payload": {
      "destination": "installed"
    }
  }
}
```

Supported `command` actions and payloads:
- `{"action": "open"}`
- `{"action": "focus"}`
- `{"action": "quit"}`
- `{"action": "status"}`
- `{"action": "navigate", "payload": {"destination": "<dest>"}}`
- `{"action": "search", "payload": {"query": "<query>"}}`
- `{"action": "view", "payload": {"mode": "<table|cards>"}}`
- `{"action": "inspect", "payload": {"package": "<pkg>"}}`
- `{"action": "inspector", "payload": {"tab": "<overview|dependencies|files>"}}`
- `{"action": "logs", "payload": {"operation": "<show|hide|clear>"}}`
- `{"action": "settings-list"}`
- `{"action": "settings-get", "payload": {"key": "<key>"}}`
- `{"action": "settings-set", "payload": {"key": "<key>", "value": "<val>"}}`
- `{"action": "settings-reset", "payload": {"key": "<key|null>"}}`

### Response Schema

```json
{
  "version": 1,
  "ok": true,
  "message": "Navigated to installed",
  "data": null,
  "error": null
}
```

On error:
```json
{
  "version": 1,
  "ok": false,
  "error": "Invalid view mode 'banana', expected 'table' or 'cards'"
}
```

### Protocol Versioning Guarantees

If a client sends a request with `version != 1`, the server immediately rejects the request with an explicit version mismatch error without modifying UI state:
```json
{
  "version": 1,
  "ok": false,
  "error": "Protocol version mismatch: client is v2, server is v1"
}
```
