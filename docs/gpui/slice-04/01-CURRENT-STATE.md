# Shelly GPUI — Slice-04 Architectural Snapshot & Component Inventory

## 1. Architectural Snapshot

Slice-04 builds directly upon the functional foundations of Slice-01 (performance primitives), Slice-02 (session & caching), and Slice-03 (dual surface & semantic inspector). It introduces a formalized GPU-accelerated motion engine and isolates high-frequency UI updates into dedicated view boundaries.

```
┌────────────────────────────────────────────────────────────────────────┐
│                              Application                               │
│                                (GPUI)                                  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                             WorkspaceView                              │
│  - AnimatedScalar (sidebar: 56px ↔ 190px)                              │
│  - AnimatedScalar (console: 0px ↔ 220px)                               │
│  - Discrete transition coordinator (dest-fade: 120ms)                  │
│  - Global keyboard navigation & toast management                       │
└───────┬───────────────────────────┬────────────────────────────┬───────┘
        │                           │                            │
        ▼                           ▼                            ▼
┌───────────────┐         ┌───────────────────┐        ┌─────────────────┐
│    Sidebar    │         │ PackageWorkstation│        │  ToastOverlay   │
│ - Compact 56px│         │ - Isolated View   │        │ - Max 3 visible │
│ - Exp. 190px  │         │ - Splitter Math   │        │ - FIFO/Err Evict│
│ - Text Clip   │         │ - Cards / Table   │        │ - 3.5s Decay    │
└───────────────┘         │ - Inspector (Tabs)│        └─────────────────┘
                          └───────────────────┘
```

---

## 2. Component Inventory & Responsibility Boundaries

### 2.1 State & Motion Primitives (`src/state/`)

| Module | Types | Primary Responsibility |
|---|---|---|
| `state::motion` | `MotionDurations`, `MotionPolicy`, `AnimatedScalar` | Standardizes animation durations (`FAST: 120ms`, `STANDARD: 160ms`, `EMPHASIS: 220ms`), quintic/sinusoidal easing curves, and provides continuous, interruptible scalar tracking. |
| `state::toast` | `ToastCenter`, `Toast`, `ToastKind`, `ToastAction`, `ToastLifecycle` | Manages transient notification messages, enforces maximum 3 visible items, deterministic FIFO eviction preserving errors, and 3.5s decay timers. |
| `state::session` | `AppSession`, `NavDestination`, `SourceFilter`, `PackageViewMode`, `InspectorTab` | Holds active UI session state with monotonic epochs (`destination_epoch`, `inspector_tab_epoch`) to trigger discrete animations. |
| `state::console` | `ConsoleModel`, `LogEntry`, `OperationStatus` | Stores terminal output streams (up to 2000 lines) with auto-scroll and disclosure state (`is_open`). |
| `state::package_store` | `PackageStore`, `PackageKey`, `UnifiedPackage` | Stores unified package inventory across ALPM, AUR, Flatpak, and AppImage with LRU session query caching. |

### 2.2 View Hierarchy (`src/views/`)

| View | Entity Type | Role & Isolation Invariant |
|---|---|---|
| `WorkspaceView` | `Entity<WorkspaceView>` | Root view. Owns navigation sidebar, active destination viewport, console log drawer, and toast overlay. Renders continuous scalar disclosures via `AnimatedScalar::update()`. |
| `PackageWorkstationView` | `Entity<PackageWorkstationView>` | Dedicated child view containing package search input, filter pills, view switcher (`Cards`/`Table`), virtualized list pane, drag splitter, and package inspector. Mouse drag movements invalidate ONLY this view, isolating high-frequency pointer updates from the root. |
| `SettingsView` | Direct View Render | Pure rendering model. Maintains an in-memory `SettingsDraft` with `is_dirty()` tracking. Contains zero disk writes during render; emits explicit save actions to persist to disk. |
| `NewsView` | Direct View Render | Displays Arch Linux news announcements. |

### 2.3 Visual Components (`src/components/`)

| Component | Responsibility |
|---|---|
| `Sidebar` | Collapsible navigation bar. Enforces strict `overflow_hidden()` on containers and text items to guarantee zero text wrapping or visual glitching during width transitions. |
| `ToastOverlay` | Fixed absolute notification container (bottom-right). Renders toast cards with kind-specific icons (`✔`, `✖`, `⚠`, `ℹ`), interactive actions (`OpenLogs`), and dismissal triggers. |
| `LogDrawer` | Operations terminal drawer. Supports continuous height interpolation from `0.0` to `220.0` px with scroll viewport clipping. |
| `PackageTable` | Virtualized 36px high-density table with 32px fixed sticky header. |
| `UnifiedSearch` | Search input bar, source filter pills, and view mode switcher. |
| `InspectorHeader` | Stationary package hero header containing title, version, badges, capability buttons, and tab bar. |
| `InspectorTab` (`Overview`, `Dependencies`, `FilesBuild`) | Discrete tab bodies wrapped in 120ms opacity fade transitions (`tab-fade`). |

---

## 3. High-Frequency UI Isolation Architecture

In earlier revisions, splitter resizing was handled directly by mutating fields on `WorkspaceView`. Because `WorkspaceView` is the parent of the navigation sidebar, settings, and drawer, any mouse move event during dragging triggered a full root rerender.

In Slice-04:
1. **Child Entity Boundary**: `PackageWorkstationView` is an independent `Entity<PackageWorkstationView>` allocated inside `cx.new(...)`.
2. **Local Pointer Capture**: Splitter mouse-down, mouse-move, and mouse-up events are attached exclusively to the workstation's internal container.
3. **Pure Math**: Pan width is computed as:
   $$\text{width} = \text{clamp}(\text{start\_width} + (\text{current\_x} - \text{start\_x}), 280.0, 700.0)$$
4. **Invalidation Scope**: When pointer events occur, only `PackageWorkstationView` calls `cx.notify()`. The root `WorkspaceView`, sidebar navigation, and bottom drawer remain completely untouched.

---

## 4. Compile-Time Invariants & Deadcode Policy

The workspace enforces zero compiler diagnostics at the crate root:

```rust
#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_must_use)]
```

No `#[allow(...)]` or lint suppression attributes are tolerated in any production code or test modules. Every struct field, enum variant, and method is actively wired to live callers.
