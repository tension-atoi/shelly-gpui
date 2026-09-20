# Shelly GPUI — Current State Baseline (Slice-06)

## Canonical Baseline

- **Slice-05 Canonical Closure SHA**: `4923dcdf06b72aeed7842d31f1c0442b099e03d3`
- **Slice-06 Target Scope**: Search Correctness, Query Workbench & Privileged Operations
- **Framework & Dependencies**:
  - `gpui = "0.2.2"` (pinned)
  - `tokio = { version = "1.40", features = ["full"] }`
  - `serde = { version = "1.0", features = ["derive"] }`
  - `serde_json = "1.0"`
  - `base64 = "0.22"`
  - `Shelly.Cli.Zig` (0.13.0 / native target)

---

## Baseline Deficiencies Identified Prior to Slice-06

1. **P0-A — Failure of Privileged Package Mutations**:
   - `Shelly.Cli.Zig` ALPM operations (`install standard`, `remove standard`, `upgrade all`) required root permissions. When executed without a controlling TTY from the GPUI backend process, mutations failed silently or aborted with permission errors because `is_terminal` returned false and elevation was bypassed.
   - Result: Users clicking "Install", "Remove", or "Upgrade All" in the GUI received immediate transaction failures without an authentication prompt.

2. **P0-B & P0-C — Protocol Parsing & Console Visibility Deficits**:
   - `Shelly.Cli.Zig` emitted base64-encoded `[JSON]...[/JSON]` frames into stdout, but `Shelly.Ui.Gpui` treated them as plain text lines, cluttering the console with raw base64 noise.
   - No structured timeline existed to show phase-level progress per package distribution backend.

3. **P1, P2, P3 — Synthetic Search Input Component**:
   - The search input was a synthetic GPUI `div()` with manual keystroke listeners. It lacked standard I-beam cursor handling, text drag selection, character boundary traversal, clipboard shortcuts, and a clear button.

4. **P4, P5, P11, P15, P16, P17 — Crowded Single-Row Filter Bar**:
   - Search input, source pills, and view mode switchers were squeezed into a single row. On narrower viewports or large package lists, elements clipped and overflowed ("Ta..." truncation).
   - No client-side sorting or state filtering existed: users could not sort by Name, Source, or Installation status.

5. **P10 — Unconstrained Splitter Squeezing**:
   - The list pane splitter clamped between 280px and 700px without checking remaining inspector width, allowing small viewports to squeeze the package inspector below its usable minimum ($< 320\text{px}$).

6. **P12, P13, P14 — Search Generation Poisoning & Sequential Search**:
   - When a user cleared the search input, `workspace.rs` invoked `set_active_results(Vec::new(), usize::MAX, cx)`. Setting `in_flight_generation` to `usize::MAX` caused all subsequent searches to fail the check `generation >= in_flight_generation`, permanently breaking search for the remainder of the session.
   - Multi-source search sequentially awaited ALPM, AUR, Flatpak, and AppImage backends, multiplying network latency.
   - Deserialization errors for Flatpak, updates, news, and AppImages were swallowed silently and converted to empty lists.
