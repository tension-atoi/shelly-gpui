# Shelly GPUI — Slice-03 Work Contract

**Slice**: SLICE-03 — Package Surface & Semantic Inspector  
**Baseline**: `main`  
**Frontend**: Rust + GPUI 0.2.2  
**Authority**: `Seafoam-Labs/Shelly-ALPM`  

======================================================================
1. PURPOSE & BOUNDARIES
======================================================================

Slice-03 turns the session, navigation, and virtualization foundations established in Slice-02 into a production-grade working surface for package management.

### Explicit Inclusions (Delivered)
1. **Dual virtualized package surface**:
   - High-density Table mode with sticky header and 36px fixed-height rows.
   - Rich Cards mode with 78px container and 72px package card.
   - Integrated view switcher toggle in the search filter bar.
   - Shared `PackageStore` results and stable `PackageKey` selection across view mode changes.
2. **Decomposed 3-tab inspector**:
   - `Overview` tab: Metadata grid, upstream links, license, sizes, install reason.
   - `Dependencies` tab: Structured dependency groups with interactive pills.
   - `Files & Build` tab: Capability-driven inspection (AUR PKGBUILD, AppImage paths, ALPM build dates, Flatpak App IDs).
   - Hero header with package identity, status badges, action buttons, copy install command, and tab switcher.
3. **Typed semantic domain model**:
   - `DependencyRef` with structured string parsing.
   - `SemanticTarget` navigation enum.
   - `PackageCapabilities` derived per package source and detail.
   - `canonical_install_command` matching Shelly CLI.
4. **Backend authority boundary & zero deadcode compliance**:
   - Full upstream compliance without unauthorized pacman wrappers.
   - `#![deny(dead_code, unused_variables, unused_imports, unused_must_use)]` with zero warnings and zero warning suppressions.

### Explicit Exclusions (Preserved for Future Slices)
- Column sorting, reordering, or user-configurable column visibility.
- Large motion or spring animation framework.
- Toast notification framework (kept simple and inline).
- Full file-tree listing for ALPM packages (as the Shelly CLI does not expose an archive content API).
- Persistent daemon or background service.

======================================================================
2. CONTRACTUAL COMMITMENTS
======================================================================

### 2.1 View Mode Invariant
- Switching between `Cards` and `Table` modes MUST NOT clear or re-fetch active search results.
- `selected_package_key` MUST remain selected across view mode switches.
- Keyboard navigation (`Up` and `Down` arrow keys) MUST navigate packages identically in both modes and maintain viewport visibility via `scroll_to_item_strict`.

### 2.2 Truthful Telemetry & Sizing
- Package sizes MUST NOT display speculative or misleading numbers.
- Uninstalled AUR packages or unmeasured packages MUST display `—` instead of `0 B` or fabricated values.
- Size formatting MUST use standard binary prefixes (KiB, MiB, GiB) with single-decimal precision.

### 2.3 Backend Authority Compliance
- All operations MUST interface through `ShellyClient` and the official Shelly CLI.
- No direct bypassing of Shelly through raw `pacman -Ql` or filesystem scraping for ALPM file contents.
- The `Files & Build` tab MUST honestly explain the capability gap for ALPM file listings rather than pretending the data is missing due to a network error.

### 2.4 Zero Deadcode Policy
- All structs, enums, methods, functions, and props created for this slice MUST be actively wired into active callers or deleted immediately under strict YAGNI.
- No `#[allow(dead_code)]`, `#[allow(unused_...)]`, or similar suppressions permitted.
