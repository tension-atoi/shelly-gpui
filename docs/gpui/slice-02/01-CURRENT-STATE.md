# Current State — Evidence Snapshot

Baseline:

    main @ e43b03ffa4aaab5d99a151d6a2491370e1f65161

This document separates established repository state from desired future state.

======================================================================
ESTABLISHED — CURRENT IMPLEMENTATION
======================================================================

## Package list

The package list has already moved from full child construction to GPUI's
`uniform_list`.

Current code uses:

    uniform_list(...)
    UniformListScrollHandle
    .track_scroll(...)

Current item geometry:

    78 px fixed item container

The virtualization closure receives only the requested visible item range and
constructs cards for that range.

This work must be preserved.

---

## Package detail cache

`WorkspaceView` currently owns:

    HashMap<String, AlpmPackage>

as:

    package_detail_cache

This avoids some duplicate package-detail subprocess queries.

However, this cache currently belongs to the root view.

Slice-02 should migrate this responsibility into an application/session data
model rather than expanding the cache inside `WorkspaceView`.

---

## Splitter

The splitter now ignores sub-pixel movement smaller than approximately 1 px.

However the current path still conceptually performs:

    pointer movement
        -> mutate WorkspaceView::list_pane_width
        -> cx.notify()
        -> WorkspaceView render

Therefore the root invalidation boundary still exists.

Do not claim that root-level resize invalidation has been architecturally
eliminated.

---

## Operation console

The console now has:

- typed `LogEntry`;
- stdout/stderr distinction;
- ANSI sanitization;
- Copy Logs;
- Clear;
- Auto-scroll toggle state;
- explicit show/hide control;
- header/body event separation.

Important current gap:

`auto_scroll_logs` currently controls presentation/toggle state, but the current
`LogDrawer` implementation does not establish an actual scroll handle that
moves the viewport to the newest streamed line.

Therefore:

    "Auto-scroll UI exists"

is established.

    "Auto-scroll behavior works"

is NOT yet established.

Slice-02 must either implement the actual behavior or stop presenting the
control as functional.

---

## Privilege elevation

Streaming mutation commands now set:

    SHELLY_ELEVATOR=pkexec

when the environment has not already provided another value.

This is downstream process configuration.

Do not broaden privilege architecture without evidence of a real blocker.

---

## Search

Search remains owned directly by `WorkspaceView`.

Current model includes:

    search_query
    search_generation
    is_searching

and dispatches directly to backend methods.

Different source tabs currently have different special-case semantics.

Examples historically include default remote searches when queries are empty.

The new unified-search model must eliminate arbitrary placeholder searches such
as using unrelated package names merely to populate a view.

---

## Workspace ownership

`WorkspaceView` still directly owns or coordinates:

- backend client;
- configuration;
- theme;
- active tab;
- search;
- package collection;
- package selection;
- package detail state;
- news;
- loading state;
- operation logs;
- operation status;
- log drawer state;
- updates count;
- splitter state;
- virtual-list scroll handle;
- package detail cache.

This has become the principal architectural constraint.

The next slice should reduce this responsibility without performing a
repository-wide rewrite.

---

## Navigation

Navigation remains a horizontal top row of source/view tabs:

    ALPM
    AUR
    Flatpak
    AppImage
    Updates
    News
    Settings

This conflates:

    application destinations

with:

    package sources

Slice-02 should separate these two concepts.

---

## Documentation evidence

No dedicated Slice-01 runtime/performance evidence record is currently present
under `docs/`.

Compilation or commit messages are not sufficient evidence of runtime frame
behavior.

Slice-02 begins with a short closure gate for this reason.
