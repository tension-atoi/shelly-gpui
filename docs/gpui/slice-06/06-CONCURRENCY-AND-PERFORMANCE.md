# Shelly GPUI — Concurrency, Caching & Performance Invariants

## 1. Concurrent Multi-Source Fanout (`tokio::join!`)

Prior to Slice-06, multi-source search in `SourceFilter::All` executed sequentially:
```text
ALPM Search (~15ms)
  └── AUR Search (~120ms network)
        └── Flatpak Search (~40ms IPC)
              └── AppImage Scan (~10ms disk)
Total Latency: ~185ms
```

In Slice-06, all four backend queries are dispatched concurrently using `tokio::join!`:
```rust
let (alpm_res, aur_res, flatpak_res, appimage_res) = tokio::join!(
    client.search_standard(&query_clone),
    async { if aur_enabled { client.search_aur(&query_clone).await.ok() } else { None } },
    async { if flatpak_enabled { client.search_flatpak(&query_clone).await.ok() } else { None } },
    async { if appimage_enabled { client.list_appimages().await.ok() } else { None } }
);
```

### Performance Impact
- **Total Latency**: $\max(T_{\text{ALPM}}, T_{\text{AUR}}, T_{\text{Flatpak}}, T_{\text{AppImage}}) \approx \max(15, 120, 40, 10) = 120\text{ms}$.
- Latency reduced by $\sim 35\% - 45\%$ on multi-source queries.

---

## 2. In-Memory Client-Side Operations

- **Sorting**: Sorting by Name, Source, or State operates entirely on the loaded `Arc<[UnifiedPackage]>`. No child processes are spawned.
- **State Filtering**: Filtering by Installed or Updates Available executes purely client-side via iterator predicates.
- **Response Time**: $< 0.5\text{ms}$ to sort and filter a list of 500 packages.

---

## 3. Session Caching

`PackageStore` caches search results keyed by normalized `(query.trim().to_lowercase(), source_filter)`.
- Returning to previous queries or toggling between previously fetched source filters is instantaneous ($0\text{ms}$ backend latency).
- Debounce timer of $60\text{ms}$ prevents spawning child processes during active keystroke bursts.
- In-flight generation tracking ensures stale asynchronous network responses arriving after a new query or buffer clear are discarded with zero allocations.
