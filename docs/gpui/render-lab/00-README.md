# Shelly Visual Render Lab

## 0. Purpose and Philosophy

The **Shelly Visual Render Lab** is an internal engineering testbed and fixture ledger designed to confront high-fidelity visual primitives against native GPU rendering capabilities.

Its core operating doctrine is:

> **No visual cleverness before the catalog, determinism model, capability vocabulary, and control surface are canonical.**

The Render Lab is **not**:
- A user-facing feature or package management view.
- A loose moodboard or collection of ad-hoc style experiments.
- A shader sandbox or arbitrary graphics playground.

The Render Lab **is**:
- A canonical fixture corpus capturing exact source designs and engineering specifications.
- A deterministic ledger tracking provenance, seeds, and capability verdicts.
- A CLI-driven verification environment built on top of the Shelly control IPC socket (protocol v2).

---

## 1. Access Model & Navigation Contract

The Render Lab is accessed strictly via developer tooling:

```bash
shelly-gpui render-lab open
```

### Invariants:
1. **Hidden from End-User Sidebar**: The Render Lab navigation destination (`NavDestination::RenderLab`) is intentionally excluded from the primary navigation rail. It does not clutter the user's package workstation.
2. **Deterministic Workspace Memory Preservation**: Entering the Render Lab does **not** overwrite `last_workspace_destination`. When the user navigates back to Browse, Installed, Updates, or News (or when settings are saved), the application preserves their working tab without accidental redirection.
3. **Offline Diagnostic Capability**: Querying `shelly-gpui render-lab status --json` while the GUI is offline safely returns a valid canonical manifest (`gui_running: false`, `catalog_count: 46`) without requiring a live Wayland session.

---

## 2. Directory Structure

```text
docs/gpui/render-lab/
├── 00-README.md               # Overview, navigation, and execution philosophy
├── 01-CANON.md                # Gnosix ratified visual axes and separation rules
├── 02-FIXTURE-CATALOG.md      # Full 46-fixture registry and provenance ledger
├── 03-CAPABILITY-LEDGER.md    # CapabilityClass promotion ladder and criteria
├── 04-DETERMINISM-CONTRACT.md # Seeds, frozen clock, and reproducible state
├── 05-ACCEPTANCE.md           # RENDER-00 sign-off criteria and gate matrix
└── references/                # Immutable source board artifacts
    ├── gnosix-board-GHIJ-source.png
    ├── gnosix-board-K-source.png
    └── SHA256SUMS
```
