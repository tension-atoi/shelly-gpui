# Slice-02 Acceptance Matrix

| Area | Test | Required Result |
|---|---|---|
| Console | Enable Auto-scroll, stream output | View follows newest lines |
| Console | Disable Auto-scroll, stream output | View position remains stable |
| Console | Copy / Clear | Existing behavior preserved |
| State | Browse -> News -> Browse | Browse session/query survives appropriately |
| State | Select package, result order changes | Selection tracks PackageKey or clears intentionally |
| Search | Type rapidly | Old request cannot replace latest results |
| Search | All query | Results from available sources merge |
| Search | One source fails | Other source results remain available |
| Search | Empty Browse | No arbitrary backend search |
| Search | Repeat identical query | Session cache may satisfy request |
| Filters | All -> AUR -> All | State/cache remains coherent |
| Installed | Open Installed | Local inventory shown without remote placeholder search |
| Updates | Complete mutation | Updates state invalidates/refreshes |
| Details | Reopen same package | Detail cache reused when valid |
| Mutation | Install/remove | Installed/detail/update caches invalidate |
| List | 1k+ package result | uniform_list remains active |
| Keyboard | Arrow through virtual list | Selected row remains visible |
| Navigation | Sidebar destination switch | No top-level source tabs remain |
| Upstream | Backend contracts | No unnecessary semantic fork |
| Build | cargo fmt/check | Pass |
| Build | release build | Pass |
| Runtime | Wayland launch | Pass |
| Regression | Polkit | Existing graphical path remains valid |
