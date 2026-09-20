# Shelly GPUI — Slice-04 Architectural Snapshot & Component Inventory

## 1. Architectural Snapshot

Slice-04 builds directly upon the functional foundations of Slice-01 (performance primitives), Slice-02 (session & caching), and Slice-03 (dual surface & semantic inspector). It introduces a formalized GPU-accelerated motion engine and isolates high-frequency UI updates and continuous frame requests into dedicated child view boundaries.

```
┌────────────────────────────────────────────────────────────────────────┐
│                              Application                               │
│                                (GPUI)                                  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                             WorkspaceView                              │
│  - Destination coordinator (dest-fade: 120ms)                          │
│  - Global keyboard navigation & active toast coordinator               │
│  - Zero continuous animation frame requests in render()                │
└───┬───────────────────┬───────────────────────────┬────────────────┬───┘
    │                   │                           │                │
    ▼                   ▼                           ▼                ▼
┌───────────────┐ ┌───────────────────┐ ┌──────────────────┐ ┌───────────────┐
│  SidebarView  │ │PackageWorkstation │ │OperationConsole  │ │ ToastOverlay  │
│ - Entity child│ │ - Entity child    │ │ - Entity child   │ │ - Max 3 items │
│ - Owns scalar │ │ - Splitter math   │ │ - Owns scalar    │ │ - FIFO/error  │
│ - Local frame │ │ - Arc<[Package]>  │ │ - Local frame    │ │ - 3.5s decay  │
│   request loop│ │ - Isolated drag   │ │   request loop   │ │ - 120ms fades │
└───────────────┘ └───────────────────┘ └──────────────────┘ └───────────────┘
```

---

## 2. Component Inventory & Responsibility Boundaries

### 2.1 State & Motion Primitives (`src/state/`)

| Module | Types | Primary Responsibility |
|---|---|---|
| `state::motion` | `MotionDurations`, `MotionPolicy`, `AnimatedScalar` | Standardizes animation durations (`FAST: 120ms`, `STANDARD: 160ms`, `EMPHASIS: 220ms`), piecewise quadratic and quintic easing curves, and provides continuous, interruptible scalar tracking. |
| `state::toast` | `ToastCenter`, `Toast`, `ToastKind`, `ToastAction`, `ToastLifecycle` | Manages transient notification messages, enforces maximum 3 visible items, deterministic FIFO eviction preserving errors, and 3.5s decay timers with 120ms entering/exiting animations. |
| `state::session` | `AppSession`, `NavDestination`, `SourceFilter`, `PackageViewMode`, `InspectorTab` | Holds active UI session state with monotonic epochs (`destination_epoch`, `inspector_tab_epoch`) to trigger discrete animations. |
| `state::console` | `ConsoleModel`, `LogEntry`, `OperationStatus` | Stores terminal output streams (up to 2000 lines) with auto-scroll and disclosure state (`is_open`). |
| `state::package_store` | `PackageStore`, `PackageKey`, `UnifiedPackage` | Stores unified package inventory across ALPM, AUR, Flatpak, and AppImage using `Arc<[UnifiedPackage]>` slices for zero-copy rendering under continuous mouse interaction. |

### 2.2 View Hierarchy (`src/views/`)

| View | Entity Type | Role & Isolation Invariant |
|---|---|---|
| `WorkspaceView` | `Entity<WorkspaceView>` | Root coordinator view. Hosts child entities (`SidebarView`, `PackageWorkstationView`, `OperationConsoleView`), routes top-level navigation, and coordinates destination entrance transitions. Contains zero continuous frame animation loops. |
| `SidebarView` | `Entity<SidebarView>` | Dedicated child entity encapsulating sidebar navigation and continuous width scalar (56px ↔ 190px). Owns its own `window.request_animation_frame()` loop, isolating continuous sidebar animation frames from root workspace. |
| `OperationConsoleView` | `Entity<OperationConsoleView>` | Dedicated child entity encapsulating drawer disclosure and continuous height scalar (0px ↔ 220px). Subscribes to console lifecycle events and drives its own `request_animation_frame()` loop exclusively during active motion. |
| `PackageWorkstationView` | `Entity<PackageWorkstationView>` | Dedicated child entity containing package search, filter pills, view switcher (`Cards`/`Table`), virtualized list pane, drag splitter, and package inspector. Mouse drag movements invalidate ONLY this view, isolating high-frequency pointer updates from the root. |
| `SettingsView` | Direct View Render | Pure rendering model. Maintains an in-memory `SettingsDraft` with `is_dirty()` tracking. Contains zero disk writes during render; emits explicit save actions to persist to disk. Clean save button is strictly inert. |
| `NewsView` | Direct View Render | Displays Arch Linux news announcements. |

### 2.3 Visual Components (`src/components/`)

| Component | Responsibility |
|---|---|
| `Sidebar` | Navigation layout rendering. Enforces strict `overflow_hidden()` on containers and text items to guarantee zero text wrapping or visual glitching during width transitions. |
| `ToastOverlay` | Fixed absolute notification container (bottom-right). Renders toast cards with kind-specific icons (`✔`, `✖`, `⚠`, `ℹ`), interactive actions (`OpenLogs`), and dismissal triggers via GPUI `with_animation`. |
| `LogDrawer` | Operations terminal drawer layout. Supports continuous height interpolation with scroll viewport clipping and terminal stream controls. |
| `PackageTable` | Virtualized 36px high-density table with 32px fixed sticky header. |
| `UnifiedSearch` | Search input bar, source filter pills, and view mode switcher. |
| `InspectorHeader` | Stationary package hero header containing title, version, badges, capability buttons, and tab bar. |
| `InspectorTab` (`Overview`, `Dependencies`, `FilesBuild`) | Discrete tab bodies wrapped in 120ms entrance fade transitions (`tab-fade`). |

---

## 3. High-Frequency UI Isolation Architecture

In earlier revisions, splitter resizing was handled directly by mutating fields on `WorkspaceView`. Because `WorkspaceView` is the parent of the navigation sidebar, settings, and drawer, any mouse move event during dragging triggered a full root rerender.

In Slice-04:
1. **Child Entity Boundary**: `PackageWorkstationView` is an independent `Entity<PackageWorkstationView>` allocated inside `cx.new(...)`.
2. **Local Pointer Capture**: Splitter mouse-down, mouse-move, and mouse-up events are attached exclusively to the workstation's internal container.
3. **Pure Math**: Pan width is computed as:
   $$\text{width} = \text{clamp}(\text{start\_width} + (\text{current\_x} - \text{start\_x}), 280.0, 700.0)$$
4. **Zero-Copy Package Slices**: `PackageStore` stores active, installed, and updates lists as `Arc<[UnifiedPackage]>`. Resizing the splitter re-renders the workstation list items via pointer clones rather than deep cloning package models.
5. **Continuous Frame Isolation**: Frame requests for animated disclosures are owned strictly by `SidebarView` and `OperationConsoleView`, preventing 60-120fps frame rendering on the root workspace.

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
