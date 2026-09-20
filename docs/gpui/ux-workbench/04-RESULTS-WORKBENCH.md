# 04 — Results Workbench & Package Identity (Phase UX-03)

## 1. Context & Executive Summary

Phase UX-02 and UX-02P established a frozen, production-grade **Command Surface** (`QueryWorkbench`), delivering a full-width search input, an unboxed resting toolbar with persistent 24px status rail, and zero floating pill chaos.

**Phase UX-03** addresses the central working surface of Shelly: the **Results Workbench**. This surface is where Linux power users, developers, and sysadmins discover, evaluate, compare, and manage packages.

### The 4 Canonical Axes of UX-03
1. **Package Identity Primitive**:
   - Deterministic resolution chain: Authenticated source-provided icon $\to$ Verified source symbolic icon $\to$ Generic package fallback.
   - **Absolute Zero Fake Logos Policy**: Never scrape the web, guess from arbitrary package names, or inject random third-party branding.
   - Stable container geometry with semantic source tinting.
2. **Information Hierarchy & Typographic Baseline**:
   - Predictable visual anchor: Name $\to$ Version $\to$ Source & Repo $\to$ Size $\to$ State.
   - Eradication of unaligned pill clusters. Truncation and alignment by deliberate design.
3. **Table as the Professional Canonical Surface**:
   - High-information density (1 row per package) as the primary workstation surface for Linux power users.
   - Strict column alignment, right-aligned numeric sizes, and inline source identity indicators.
   - Clean installation default: `Table` view mode by default for new installations, without clobbering existing `gpui-ui.json` configurations.
4. **Cards as a Genuinely Distinct View**:
   - Justify the 72px–80px spatial footprint through rich identity (36x36 avatar), multi-line context-rich descriptions (2 lines in normal mode), and tactile selection affordance.
   - A Card is not a table row wrapped in a border; it is an evaluation canvas.

---

## 2. Strict Scope Boundaries

To maintain software integrity and avoid regressions:
- **Command Surface is FROZEN**: `query_workbench.rs`, `search_input.rs`, `menu.rs`, and `view_mode_switcher.rs` are untouched.
- **Inspector is OUT OF SCOPE**: Detailed tabs, dependencies, files, and build scripts belong strictly to **Phase UX-04**.
- **Zero Deadcode Policy**: Strict enforcement of `#![deny(dead_code)]`, `#![deny(unused_variables)]`, `#![deny(unused_imports)]`, and `#![deny(unused_must_use)]`.

---

## 3. Axis 1: Package Identity Primitive

### 3.1 The Problem with Naive Package Displays
In naive software centers, packages either display no visual anchor at all (monotonous walls of text) or commit the fatal flaw of "fake logos" (heuristic web scraping, downloading random low-resolution images, or mapping package name prefixes to arbitrary brand assets).

### 3.2 The Strict Identity Resolution Chain
Shelly enforces a truthful, three-tier resolution chain:

```mermaid
flowchart TD
    Start["Package Item Evaluation"] --> Step1{"Authentic Source Icon Provided?<br/>(e.g., AppImage icon_name, desktop file)"}
    Step1 -- Yes --> UseProvided["1. Authentic Source Icon"]
    Step1 -- No --> Step2{"Known Package Source?<br/>(ALPM, AUR, Flatpak, AppImage)"}
    Step2 -- Yes --> UseSymbolic["2. Verified Source Symbolic SVG<br/>(Arch Swoosh, AUR Crest, Flatpak Cube, AppImage Diamond)"]
    Step2 -- No --> UseGeneric["3. Generic Package Fallback SVG<br/>(Neutral Package Box)"]
```

### 3.3 Semantic Color Tinting
Each source is assigned a distinct, high-contrast semantic identity token:

| Package Source | Symbolic Glyph | Dark Theme Tint (`bg / border`) | Light Theme Tint (`bg / border`) | Semantic Meaning |
| :--- | :--- | :--- | :--- | :--- |
| **ALPM** | Arch Swoosh | `rgb(0x38bdf8)` @ 12% / 25% | `rgb(0x0284c7)` @ 10% / 20% | Official Arch Repositories (`core`, `extra`, `multilib`) |
| **AUR** | AUR Community Crest | `rgb(0xa855f7)` @ 12% / 25% | `rgb(0x7c3aed)` @ 10% / 20% | Arch User Repository (Community PKGBUILDs) |
| **Flatpak** | Isometric Cube | `rgb(0x06b6d4)` @ 12% / 25% | `rgb(0x0891b2)` @ 10% / 20% | Sandboxed Flathub / Remote Runtimes |
| **AppImage** | Portable Diamond | `rgb(0xf97316)` @ 12% / 25% | `rgb(0xea580c)` @ 10% / 20% | Self-contained Executable Binaries |
| **Generic** | Package Box | `border` @ 50% | `border` @ 50% | Unspecified / Fallback |

---

## 4. Axis 2 & 4: Cards View Architecture

### 4.1 Card Geometry & Spatial Budget
- **Normal Mode**:
  - Row Wrapper Height: `80.0px` (`UiMetrics::CARD_WRAPPER_NORMAL`)
  - Inner Card Height: `72.0px` (`UiMetrics::CARD_HEIGHT_NORMAL`)
  - Vertical Inset: `4.0px` top and bottom (`py(px(4.0))`), perfectly matching $72 + 4 + 4 = 80\text{px}$.
  - Avatar Dimensions: `36.0px` $\times$ `36.0px` with `6.0px` rounded corners.
- **Compact Mode**:
  - Row Wrapper Height: `66.0px` (`UiMetrics::CARD_WRAPPER_COMPACT`)
  - Inner Card Height: `58.0px` (`UiMetrics::CARD_HEIGHT_COMPACT`)
  - Vertical Inset: `4.0px` top and bottom (`py(px(4.0))`), perfectly matching $58 + 4 + 4 = 66\text{px}$.
  - Avatar Dimensions: `28.0px` $\times$ `28.0px` with `4.0px` rounded corners.

### 4.2 Card Visual Layout

```text
┌─[4px Accent]────────────────────────────────────────────────────────────────────────┐
│  ┌──────────┐  ripgrep                              14.1.0-1          1.8 MiB       │
│  │  [ALPM]  │  [ALPM · extra]  [Installed]                                          │
│  │  Swoosh  │  Ultra-fast line-oriented search tool combining the usable ergonomics │
│  └──────────┘  of ag with the raw speed of grep.                                    │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

1. **Left Visual Anchor**:
   - The `PackageIdentity` avatar occupies a fixed square container.
   - When selected, a `4px` electric cyan accent rail anchors the left edge.
2. **Right Content Column**:
   - **Line 1 (Header)**: Package Name in `FontWeight::BOLD`, `text_sm`, with `theme.text_primary` (or `theme.accent` when selected). The right side presents the Version (`text_xs`, `text_muted`) and Size if known.
   - **Line 2 (Metadata Rail)**: Crisp, non-colliding badges: Source + Repository (`ALPM · extra`), and State (`Installed`, `Update Available`, or neutral `Available`).
   - **Line 3 (Description)**: In Normal mode, allows 2 clean lines of secondary text (`text_xs`, `theme.text_secondary`), giving real substance to search results. In Compact mode, cleanly clamped to 1 line.

---

## 5. Axis 2 & 3: Table View Architecture

### 5.1 The Table as Canonical Workstation Surface
For power users managing thousands of system packages, the Table view is the primary workhorse. It maximizes vertical scanning speed and provides instantaneous comparison of versions, repositories, and disk footprints.

### 5.2 Column Topology & Metrics
Row Height:
- Normal Mode: `36.0px` (`UiMetrics::ROW_HEIGHT_NORMAL`)
- Compact Mode: `30.0px` (`UiMetrics::ROW_HEIGHT_COMPACT`)
- Header Height: `32.0px` (`UiMetrics::ROW_HEIGHT_HEADER`)

| Column | Header | Proportion / Width | Content Alignment | Visual Elements |
| :--- | :--- | :--- | :--- | :--- |
| **Col 1** | `NAME` | `flex_1` (expandable) | Left | 16x16 inline `PackageIdentity` avatar + Package Name |
| **Col 2** | `VERSION` | `110.0px` fixed | Left | Monospace/clean version string (`text_xs`, `text_muted`) |
| **Col 3** | `SOURCE` | `90.0px` fixed | Left | Source badge (`ALPM`, `AUR`, `Flatpak`, `AppImage`) |
| **Col 4** | `SIZE` | `75.0px` fixed | Right (`justify_end`) | Formatted disk size (`format_bytes`) or neutral `—` |
| **Col 5** | `STATUS` | `85.0px` fixed | Left / Center | Semantic state badge (`Installed`, `Update`, `Available`) |

### 5.3 Table Header & Row Synchronization
Header columns and row cells share the exact same width tokens (`110px`, `90px`, `75px`, `85px`) and padding (`px_3`), guaranteeing zero horizontal jitter or columnar misalignment.

---

## 6. View Mode Configuration & Clean Install Default

### 6.1 Configuration Persistence in `gpui-ui.json`
`PackageViewMode` is integrated directly into `GpuiUiConfig`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuiUiConfig {
    // ...
    #[serde(default = "default_view_mode")]
    pub view_mode: PackageViewMode,
}

fn default_view_mode() -> PackageViewMode {
    PackageViewMode::Table
}
```

### 6.2 Strict Non-Regression Invariants
1. **Clean Installation**: When no `~/.config/shelly/gpui-ui.json` exists, Shelly launches in **Table** mode as the canonical professional surface.
2. **Existing Configuration Preservation**: If an existing `gpui-ui.json` is present without the `view_mode` field, Serde safely defaults to `Table`. If the user has explicitly selected `Cards`, their choice is loaded and preserved.
3. **Reactive Persistence**: Switching view modes via the toolbar `ViewModeSwitcher` immediately saves the new preference to `gpui-ui.json`.

---

## 7. Quality Gates & Acceptance Matrix

| Gate | Requirement | Target |
| :--- | :--- | :--- |
| **Unit Tests** | `cargo test --release --locked` | 100% passing, zero regressions |
| **Linter** | `cargo clippy --release --locked -- -D warnings` | 0 warnings, zero deadcode |
| **Packaging** | `makepkg -C -c -f` | Arch Linux native package built |
| **Wayland Verification** | Hyprland live runtime | Screenshots verifying Cards & Table view modes |
