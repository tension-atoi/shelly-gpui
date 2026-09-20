# Shelly GPUI — Slice-05 Work Contract

## 1. Scope & Objective

**SLICE-05 — Visual System, Desktop Adaptation, Settings IA & Product Finishing**

The objective of Slice-05 is to complete all visual and desktop integration contracts for Shelly GPUI, aligning the application with production-grade Linux desktop standards.

---

## 2. Invariant Commitments & Core Rules

### 2.1 Zero-Deadcode Policy & No Warning Suppression
- In accordance with workspace standards (`/home/tension_atoi/AGENTS.md`):
  - `#![deny(dead_code)]`
  - `#![deny(unused_variables)]`
  - `#![deny(unused_imports)]`
  - `#![deny(unused_must_use)]`
- **Zero dead code**: No struct, enum variant, field, or method is permitted to exist without being actively wired to live callers.
- **Zero compiler or linter warning suppression**: No `#[allow(...)]`, `#[allow(dead_code)]`, or `#[allow(clippy::...)]` attributes. All code must compile cleanly under standard flags and `cargo clippy -- -D warnings`.

### 2.2 Strictly Pinned Dependencies
- `gpui = "0.2.2"` is strictly pinned in `Cargo.toml`.
- All features must compile against the GPUI 0.2.2 API surface without relying on unreleased or unversioned APIs.

### 2.3 Configuration Separation of Authority
- `~/.config/shelly/settings.json` is the sole file owned and managed by the GPUI frontend.
- `~/.config/shelly/config.json` is owned by the Zig CLI runtime.
- The GPUI frontend must never overwrite, truncate, or confuse the schemas of these two independent configuration authorities.

### 2.4 Mathematical Splitter Invariant
- The splitter calculation must enforce:
  $$\text{Inspector Width} \ge \text{INSPECTOR\_MIN\_WIDTH} = 320.0\,\text{px}$$
  across all supported window widths ($\ge 1024.0\,\text{px}$).
- Formula:
  $$\text{dynamic\_list\_max} = \min(700.0,\, \text{window\_width} - \text{SIDEBAR\_EXPANDED} - \text{SPLITTER\_WIDTH} - \text{INSPECTOR\_MIN\_WIDTH})$$
  $$\text{final\_width} = \text{clamp}(\text{requested\_width},\, \text{LIST\_MIN\_WIDTH},\, \max(\text{LIST\_MIN\_WIDTH},\, \text{dynamic\_list\_max}))$$

### 2.5 Single Busy Execution Authority
- `ConsoleModel::is_running()` is the single source of truth for process execution state.
- Redundant booleans (e.g. `WorkspaceView.is_mutating`) are strictly prohibited.
- Views must not subscribe to high-frequency log stream events (`LogAppended`) for execution state tracking; only lifecycle events (`OperationStarted`, `OperationFinished`) may trigger notification.

### 2.6 Honest Settings & Actionable Options
- Every toggle displayed in `SettingsView` must directly control live functionality in the GPUI application or downstream Zig CLI execution.
- Ghost settings that have no operational effect are strictly removed from the UI.
- Modifying settings must not perform disk writes until the user explicitly clicks "Save Settings".

---

## 3. Scope Boundaries

### What Slice-05 Delivers:
1. `ui_metrics.rs` centralizing density and layout geometry.
2. 22 SVG icons in `icons/` embedded at compile-time via `include_bytes!`.
3. Reorganization of `SettingsView` into 5 logical categories with dirty tracking and reset action.
4. Active wiring of source enablement booleans to search queries and filter pills.
5. Active wiring of cascade deletion and configuration removal flags to `remove_package`.
6. Sampling of window geometry at Save time without continuous resize listeners.
7. Geometry sanitization ensuring window dimensions are strictly $\ge 1024 \times 680$ px.
8. Packaging hardening with strict `cargo test --release` in `PKGBUILD-gpui`.
9. Interactive Arch Linux news announcements with browser link-out via `cx.open_url`.
10. Full Dark and Light theme parity and automated contrast verification.

### What is Explicitly Out of Scope for Slice-05:
- In-memory package database caching across application launches (persisted cache).
- Multi-window or detached inspector workflows.
- Custom user theme editor or CSS customization engine.
- Direct root-level ALPM library C-bindings (all mutations flow through the Zig CLI authority).
