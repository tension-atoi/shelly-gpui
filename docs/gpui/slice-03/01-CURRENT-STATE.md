# Current State — Evidence Snapshot & Architecture Inventory

**Slice**: SLICE-03 — Package Surface & Semantic Inspector  
**Baseline**: `main`  
**Frontend**: GPUI 0.2.2  

======================================================================
ESTABLISHED ARCHITECTURE INVENTORY
======================================================================

## 1. Application State Architecture (`src/state/`)

- **`AppSession`** (`session.rs`):
  - Retained GPUI entity holding user navigation, view modes, search state, and package selection.
  - Fields:
    - `destination: NavDestination` (`Browse`, `Installed`, `Updates`, `News`, `Settings`)
    - `source_filter: SourceFilter` (`All`, `Alpm`, `Aur`, `Flatpak`, `AppImage`)
    - `search_query: String`
    - `selected_package_key: Option<PackageKey>`
    - `search_generation: usize` (monotonic generation counter for race prevention)
    - `is_searching: bool`
    - `sidebar_collapsed: bool`
    - `view_mode: PackageViewMode` (`Cards`, `Table`)
    - `inspector_tab: InspectorTab` (`Overview`, `Dependencies`, `FilesBuild`)
  - Typed events (`SessionEvent`):
    - `DestinationChanged`, `SourceFilterChanged`, `SearchQueryChanged`, `PackageSelected`, `SidebarToggled`, `ViewModeChanged`, `InspectorTabChanged`.

- **`PackageStore`** (`package_store.rs`):
  - Retained GPUI entity managing package data, caches, and invalidations.
  - Fields:
    - `client: ShellyClient`
    - `active_results: Vec<UnifiedPackage>`
    - `active_generation: usize`
    - `search_cache: HashMap<SearchKey, Vec<UnifiedPackage>>`
    - `installed_packages: Vec<UnifiedPackage>`
    - `updates_packages: Vec<UnifiedPackage>`
    - `updates_count: usize`
    - `detail_cache: HashMap<PackageKey, AlpmPackage>`
    - `pkgbuild_cache: HashMap<String, String>` (stores AUR PKGBUILD recipes fetched on demand)
  - Invalidation:
    - `invalidate_package(&key)` purges both ALPM detail cache and AUR PKGBUILD cache.
    - `invalidate_installed()` and `invalidate_updates()` reset local package sets.

- **`Semantic Models`** (`semantic.rs`):
  - `DependencyKind`: `Runtime`, `Build`, `Optional`, `RequiredBy`.
  - `DependencyRef`: Structured parsing of dependency strings into `name`, optional `constraint` (e.g. `>=14`), and optional `description`.
  - `SemanticTarget`: Enum representing navigation and interaction destinations (`Package`, `ExternalUrl`, `FilePath`, `CopyText`).
  - `PackageCapabilities`: Structured capability flags derived per package: `can_install`, `can_remove`, `can_update`, `has_pkgbuild`, `has_file_tree`, `has_dependencies`.
  - `canonical_install_command(pkg)`: Generates Shelly CLI install invocations adhering strictly to CLI syntax (`shelly install standard/aur/flatpak <name>`).

- **`ConsoleModel`** (`console.rs`):
  - Tracks live execution stream, auto-scroll status (`ScrollHandle`), and operation lifecycles (`Idle`, `Running`, `Success`, `Error`).

---

## 2. Presentation & Component Inventory (`src/components/`, `src/views/`)

- **`PackageTable`** (`components/package_table.rs`):
  - Sticky header (`render_header`): Fixed 32px height with columns `NAME` (flex-1), `VERSION` (100px), `SOURCE` (80px), `SIZE` (70px), `STATUS` (75px).
  - Virtualized row (`render_row`): Fixed 36px height with high-contrast text, source pills, formatted sizes, and status badges.
  - Truthful size formatting: `format_bytes` (B, KiB, MiB, GiB) and `display_size` distinguishing uninstalled AUR packages (`—`) from measured packages.

- **`UnifiedSearch`** (`components/unified_search.rs`):
  - Search bar input with live debouncing (60ms).
  - Source filter pills (`All`, `Official / ALPM`, `AUR`, `Flatpak`, `AppImage`).
  - Result count telemetry label.
  - View switcher pill toggle: `▦ Cards` and `☰ Table`.

- **`InspectorHeader`** (`components/inspector_header.rs`):
  - Hero header with package name, version, source badge, status pill, and repository.
  - Capability-driven action buttons (Install/Remove/Update).
  - Canonical install command copy button with 2s visual confirmation feedback ("Copied!").
  - Tab switcher bar for `Overview`, `Dependencies`, `Files & Build`.

- **`PackageInspectorView`** (`views/inspector/`):
  - Root coordinator rendering `InspectorHeader` and active tab view.
  - `OverviewTab` (`overview.rs`): Metadata grid with upstream URLs, licenses, package sizes, maintainer, install reason.
  - `DependenciesTab` (`dependencies.rs`): Groups runtime dependencies, make dependencies, optional dependencies, and reverse required-by packages into interactive `DependencyRef` pill grids with click-to-search.
  - `FilesBuildTab` (`files_build.rs`): Displays live AUR `PKGBUILD` scripts with monospaced styling and copy button; AppImage local paths and desktop files; ALPM metadata with explicit notice regarding Shelly CLI file tree constraints; Flatpak application IDs.

- **`SemanticValue`** (`components/semantic_value.rs`):
  - Reusable element for clickable URLs (`cx.open_url`), copyable values (`cx.write_to_clipboard`), and inter-package navigation.

- **`WorkspaceView`** (`views/workspace.rs`):
  - Wires dual view modes (`Cards` vs `Table`), `UniformListScrollHandle`, keyboard navigation (`Up`/`Down`/`Escape`), splitter resizing, and reactive session event subscriptions.
