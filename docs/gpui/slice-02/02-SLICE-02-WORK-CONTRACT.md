# WORK CONTRACT — SHELLY GPUI / SLICE-02

## Session Architecture, Workstation Navigation & Unified Search

STATUS:

    Execute only after explicit Operator GO.

MODE:

    Bounded implementation slice.

Do not start Phase-03 visual expansion during this work.

======================================================================
1. PURPOSE
======================================================================

The previous slice established the first performance primitives:

- virtualized package rendering;
- package-detail session caching;
- better console interaction;
- graphical privilege elevation.

The next architectural problem is state ownership.

Shelly GPUI currently treats the root `WorkspaceView` as:

- application controller;
- cache;
- navigation model;
- search model;
- package store;
- operation console host;
- layout owner.

This is not sustainable once the UI gains:

- a sidebar;
- unified search;
- source filters;
- table/card modes;
- inspector subviews;
- transitions;
- richer semantic navigation.

Slice-02 must establish a proper session/state layer before those consumers
multiply.

======================================================================
2. DESIGN PRINCIPLE
======================================================================

Adopt the tty7-style architectural lesson without copying tty7's daemon model:

    backend authority
        !=
    session state
        !=
    presentation

Target conceptual flow:

    Shelly CLI / backend
            ↓
       PackageStore
            ↓
        AppSession
            ↓
     GPUI surfaces

The frontend remains native Rust/GPUI.

No daemon is introduced.

======================================================================
3. GPUI STATE MODEL
======================================================================

Use GPUI's current retained entity model.

Expected primitives:

    Entity<T>
    Context<T>
    cx.notify()
    cx.observe()
    cx.subscribe()
    cx.emit()

Do not simulate a React-style global store with `Rc<RefCell<_>>`.

Do not introduce Redux-like infrastructure.

Keep the model small.

Recommended minimum separation:

    AppSession
    PackageStore
    WorkspaceView

### AppSession

Owns application/session intent:

- current destination;
- active Browse source filter;
- active search query;
- selected package identity;
- loading/search generation metadata where UI-global;
- navigation state required to restore user context.

### PackageStore

Owns package-domain presentation data:

- active result sets;
- installed-package result set;
- update count/data;
- package-detail cache;
- session search cache;
- in-flight request generations;
- package data invalidation.

### WorkspaceView

Owns ephemeral rendering/layout state only where reasonable:

- focus handles;
- pane/split dimensions;
- scroll handles;
- pointer-drag transient state;
- local presentation composition.

Operation-console extraction may be deferred unless naturally required.

Do not create ten entities because three are sufficient.

======================================================================
4. DOMAIN EVENTS
======================================================================

Use typed events where they reduce coupling.

Examples:

    SessionEvent::DestinationChanged
    SessionEvent::SearchChanged
    SessionEvent::SourceFilterChanged

    PackageStoreEvent::ResultsChanged
    PackageStoreEvent::PackageInvalidated
    PackageStoreEvent::UpdatesChanged

Do not emit events redundantly when direct observation is simpler.

`cx.observe()` is appropriate when a consumer cares that another entity's
state changed.

`cx.subscribe()` / `cx.emit()` is appropriate for semantic events.

======================================================================
5. PACKAGE IDENTITY
======================================================================

Introduce a stable presentation-level package identity instead of treating a
list index as the durable identity of the selected package.

Minimum identity must distinguish source.

Example conceptual type:

    PackageKey {
        source,
        name,
        remote/repository where necessary
    }

Do not use list index as the canonical package identity outside viewport-local
selection mechanics.

Reason:

Search result reordering, filtering and source merging make indexes unstable.

The virtual list may still address visible items by index.

The selected domain package must not.

======================================================================
6. PHASE 0 — CLOSE SLICE-01 GAPS
======================================================================

Before architectural expansion:

### 6.1 Real console auto-scroll

Implement actual auto-scroll behavior.

Requirements:

- when enabled and streamed output arrives, the log viewport follows the newest
  content;
- when disabled, new lines do not force viewport movement;
- toggling back on resumes follow behavior;
- manual inspection is possible.

Use an appropriate GPUI scroll handle or equivalent supported primitive.

Do not leave a toggle that changes only its label.

### 6.2 Runtime evidence note

Create:

    docs/gpui/slice-01-performance-evidence.md

Record what can actually be measured now.

At minimum:

- baseline commit;
- release/debug mode;
- package count tested;
- splitter behavior;
- virtual list behavior;
- known limitations.

Do not invent an FPS measurement if no measurement facility is available.

Use:

    MEASURED
    OBSERVED
    ESTABLISHED
    INFERENCE

explicitly.

======================================================================
7. WORKSTATION NAVIGATION
======================================================================

Replace the source-oriented horizontal navigation with a left workstation
sidebar.

Top-level destinations:

    Browse
    Installed
    Updates
    News
    Settings

Updates retains a count badge.

Package sources must NOT remain top-level application destinations.

They become filters within Browse.

### Browse

Search/discovery across package sources.

### Installed

Local package inventory.

### Updates

Available updates.

### News

Arch/Shelly news surface.

### Settings

Application configuration.

Do not add additional destinations during this slice.

======================================================================
8. SIDEBAR VISUAL CONTRACT
======================================================================

Maintain the emerging Zed/Shadcn-influenced language.

Characteristics:

- dark, quiet navigation rail;
- compact rows;
- crisp active state;
- one accent color;
- restrained badges;
- no oversized icons;
- no decorative gradients;
- no visual noise.

Use current theme tokens where possible.

Do not perform a full design-system rewrite.

Target desktop ergonomics, not mobile navigation.

No animated sidebar collapse in this slice.

======================================================================
9. UNIFIED SEARCH
======================================================================

Browse owns one search surface.

Search filters:

    All
    ALPM
    AUR
    Flatpak
    AppImage

The filter is a search source selector, not a route.

### Query semantics

A non-empty query triggers search.

An empty Browse query must NOT issue arbitrary placeholder remote searches.

Do not use:

    "git"
    "browser"

or another unrelated search term merely to populate the UI.

For an empty Browse query, show an intentional empty/discovery state.

### All

For a non-empty query:

- query the supported sources;
- execute independent source queries concurrently where safe;
- merge only after associating each result with its source;
- preserve deterministic ordering;
- preserve errors per source without failing the entire unified result surface.

Example:

    ALPM succeeds
    AUR succeeds
    Flatpak unavailable

must still show ALPM + AUR results and expose Flatpak failure unobtrusively.

Do not silently drop source failures.

======================================================================
10. SEARCH SCHEDULING
======================================================================

Search text echo must remain immediate.

Backend search execution should remain debounced/coalesced.

Do not assume the current 50 ms value is optimal.

The relevant requirement is:

    typing must feel immediate
    while subprocess churn remains bounded

Maintain a generation/request identity mechanism.

A stale request must never overwrite results belonging to a newer query.

When implementing All-source search, request identity must account for:

    query
    source filter
    destination/session context

======================================================================
11. SESSION SEARCH CACHE
======================================================================

Add a small in-memory session cache.

Conceptual key:

    SearchKey {
        query,
        source_filter
    }

Value:

    search results
    optional freshness metadata

No disk persistence.

No SQLite.

No database.

No serialized package catalog.

The cache exists solely to avoid immediately repeating identical backend work
within one application session.

Switching:

    All -> AUR -> All

should be able to reuse already available results when valid.

======================================================================
12. INVALIDATION
======================================================================

Cache correctness outranks cache hit rate.

At minimum invalidate relevant data after:

- successful package install;
- successful package removal;
- successful upgrade;
- explicit refresh;
- repository synchronization if exposed.

Invalidate:

- affected package details;
- Installed data;
- Updates data;
- affected active search results where state/status could have changed.

Do not clear every cache for every operation unless necessary.

Document the chosen invalidation rules.

======================================================================
13. INITIAL LOAD
======================================================================

Current initialization serializes some independent operations.

Review whether independent startup queries such as:

    updates count
    installed packages

can execute concurrently.

Do not block initial usable UI on unrelated remote sources.

Desired UX:

    shell renders immediately
    local state appears incrementally
    remote state fills asynchronously

Do not introduce a splash screen.

======================================================================
14. SELECTION & NAVIGATION CONTINUITY
======================================================================

When possible, preserve useful session context.

Examples:

- returning from package detail state does not unnecessarily forget Browse query;
- switching source filter does not destroy cached results;
- package selection uses `PackageKey`;
- when the selected package disappears from a result set, clear selection
  intentionally;
- keyboard up/down navigation keeps the virtualized item visible using
  `UniformListScrollHandle`.

Current GPUI documentation explicitly supports:

    scroll_to_item(...)
    scroll_to_item_strict(...)
    scroll_to_item_with_offset(...)

Use these rather than manually estimating viewport offsets.

======================================================================
15. SEMANTIC INTERACTION FOUNDATION
======================================================================

Do not implement the full tty7-style semantic inspector yet.

But structure package data so future interactions can address:

- dependency package;
- repository;
- upstream URL;
- file path;
- conflict;
- provided capability;

as typed semantic targets.

Do not bake these interactions into unstructured display strings.

This is preparation only.

No context-menu framework is required in this slice.

======================================================================
16. SPLITTER STATE
======================================================================

Do not worsen the current resize path.

If extracting `WorkspaceView` state naturally allows splitter layout state to
be isolated into a smaller rendering entity, do it.

Otherwise preserve current behavior and document it as residual work.

Do not start a broad docking-layout implementation.

The sidebar itself does not need to be resizable in this slice.

======================================================================
17. UPSTREAM CONTRACT
======================================================================

Prefer frontend/session changes only.

Do not modify Shelly Zig backend semantics unless required to expose data the
existing JSON contract genuinely lacks.

If a backend change appears necessary:

STOP.

Document:

- missing information;
- current command;
- desired contract;
- why presentation-side derivation is insufficient.

Do not silently fork backend semantics.

======================================================================
18. NON-GOALS
======================================================================

Explicitly OUT OF SCOPE:

- gpui-kit dependency adoption;
- GPUI version migration;
- React / GPUIX;
- JavaScript runtime;
- daemon;
- SQLite;
- persistent package cache;
- DataTable;
- table/card toggle;
- inspector tabs;
- Files/PKGBUILD inspector implementation;
- dependency graph;
- toast system;
- generalized animation framework;
- page slide transitions;
- semantic context menus;
- agent awareness;
- full English localization sweep.

New code and new UI strings should be written in English.

Legacy French strings may remain until the localization slice.

======================================================================
19. FILE / MODULE DIRECTION
======================================================================

Exact names may vary, but prefer an explicit structure similar to:

    src/
      state/
        mod.rs
        session.rs
        package_store.rs

      components/
        sidebar.rs
        unified_search.rs
        package_card.rs
        log_drawer.rs

      views/
        workspace.rs
        details.rs
        news.rs
        settings.rs

Do not perform mechanical file splitting with no ownership improvement.

A module extraction is valid only if responsibility actually moves with it.

======================================================================
20. ACCEPTANCE
======================================================================

Slice-02 is complete only when:

- actual log auto-scroll works;
- Slice-01 evidence note exists;
- package/session domain state is no longer principally owned by WorkspaceView;
- PackageStore is a GPUI entity or equivalent clean entity boundary;
- selected packages have stable source-aware identity;
- sidebar exists with exactly:
  Browse / Installed / Updates / News / Settings;
- source filters exist within Browse:
  All / ALPM / AUR / Flatpak / AppImage;
- empty Browse does not issue fake/default remote searches;
- All search combines supported source results;
- stale async search results cannot overwrite newer state;
- repeated identical searches can reuse in-session data;
- mutation invalidation is documented and implemented;
- virtualized package list remains intact;
- keyboard navigation keeps selected virtual rows visible;
- install/remove/upgrade functionality still works;
- console functionality still works;
- graphical Polkit path still works;
- cargo fmt passes;
- cargo check passes;
- release build succeeds;
- Wayland runtime validation is performed.

======================================================================
21. COMMIT DISCIPLINE
======================================================================

Prefer coherent commits such as:

    fix(console): implement real follow-mode scrolling

    refactor(state): introduce AppSession and PackageStore entities

    refactor(packages): migrate detail cache and package selection identity

    feat(nav): add workstation sidebar destinations

    feat(search): add source-aware unified search and session cache

    docs: record slice-01 evidence and slice-02 architecture

Do not combine Phase-03 features into these commits.

======================================================================
22. DEFINITION OF DONE
======================================================================

This slice is not done because the sidebar looks good.

It is done when the architecture has changed from:

    WorkspaceView owns application

to:

    WorkspaceView renders application state owned by bounded GPUI entities.

The visible sidebar and unified search are the first consumers proving that the
new architecture works.

======================================================================
23. NEXT VERSION — DO NOT EXECUTE
======================================================================

After acceptance, the intended next slice is:

    SLICE-03 — Package Surface & Semantic Inspector

Likely scope:

- card / DataTable switcher;
- high-density table;
- Overview inspector;
- Dependencies inspector;
- Files & Build inspector;
- typed semantic links;
- contextual actions;
- better package hero/action area.

Motion remains a separate concern unless Slice-03 proves that a small shared
transition primitive is required.

Do not begin Slice-03 without explicit Operator GO.
