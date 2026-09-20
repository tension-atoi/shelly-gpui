# Shelly GPUI — Search Architecture & Input Mechanics

## Overview

In Slice-06, search in Shelly GPUI was elevated from a prototype keystroke listener to a native, full-featured text-editing component backed by a robust, asynchronous query engine.

---

## 1. Native `SearchInput` Component Architecture

`Shelly.Ui.Gpui/src/components/search_input.rs` defines `SearchInputView`, an `Entity` that implements GPUI's native input contracts:

```rust
impl EntityInputHandler for SearchInputView {
    fn text_for_range(&mut self, range_utf16: Range<usize>, _cx: &mut Context<Self>) -> Option<String>;
    fn selected_text_range(&mut self, _cx: &mut Context<Self>) -> Option<UTF16Selection>;
    fn marked_text_range(&self, _cx: &mut Context<Self>) -> Option<Range<usize>>;
    fn unmark_text(&mut self, _cx: &mut Context<Self>);
    fn replace_text_in_range(&mut self, replacement_range_utf16: Option<Range<usize>>, text: &str, cx: &mut Context<Self>);
    fn replace_and_mark_text_in_range(&mut self, range_utf16: Option<Range<usize>>, new_text: &str, new_selected_range_utf16: Option<Range<usize>>, cx: &mut Context<Self>);
    fn bounds_for_range(&mut self, range_utf16: Range<usize>, bounds: Bounds<Pixels>, cx: &mut Context<Self>) -> Option<Bounds<Pixels>>;
    fn character_index_for_point(&mut self, point: Point<Pixels>, bounds: Bounds<Pixels>, cx: &mut Context<Self>) -> Option<usize>;
}
```

### Pointer Interactions
- **I-Beam Cursor**: Hovering the input displays the standard text editing `CursorStyle::IBeam`.
- **Caret Placement**: Clicking samples the horizontal pointer position against character width metrics to position the caret precisely.
- **Drag Selection**: Clicking and dragging dynamically updates the selection range, with visual selection highlight rendering.

### Keyboard Navigation & Shortcuts
- **Character Insertion**: Characters typed at the caret replace any active selection and advance the caret.
- **Backspace & Delete**: Respect active selections; when no selection exists, delete the preceding or succeeding character while respecting UTF-8 boundaries.
- **Arrow Keys**: Left and Right move the caret; Shift+Left/Right expands or contracts the selection.
- **Home & End**: Jump to the beginning or end of text; Shift+Home/End selects to boundary.
- **Ctrl+A**: Selects the entire text buffer.
- **Escape**: Clears the search buffer immediately while preserving focus.

### Clear Button & Search Pulse
- A vector `[×]` button (`AppIcon::Close`) appears on the right edge whenever the input is non-empty.
- Clicking or triggering Enter/Space on the clear button resets the buffer and maintains focus.
- When an asynchronous search query is in-flight, an animated pulsing indicator beside the clear button provides immediate visual feedback.

---

## 2. Search Debounce & Execution Flow

To balance instantaneous local responsiveness with minimal backend load:
1. **Debounce Interval**: 60ms timer via `cx.background_executor().timer(Duration::from_millis(60))`.
2. **Session Cache Check**: Prior to launching CLI child processes, `PackageStore::get_cached_search(&query, filter)` checks if an identical query/filter pair exists in the current session.
3. **Monotonic Generation Allocation**: Every search query allocates `next_search_generation()`.
4. **Concurrent Execution**: Dispatched as an asynchronous task via `cx.spawn(...)`.

---

## 3. Elimination of Generation Poisoning

### The Previous Bug
Prior to Slice-06, clearing an empty query executed:
```rust
// BUGGY PATTERN (REMOVED):
self.store.update(cx, |st, cx| {
    st.set_active_results(Vec::new(), usize::MAX, cx);
});
```
`in_flight_generation` was set to `usize::MAX`. Any subsequent search returned a generation of $N \ll \text{usize::MAX}$, failing the guard `generation >= self.in_flight_generation`, and was permanently dropped.

### The Canonical Fix
```rust
// CANONICAL FIX IN SLICE-06:
let next_gen = self.session.update(cx, |s, cx| {
    s.search_query.clear();
    s.set_searching(false, cx);
    s.select_package(None, cx);
    s.next_search_generation()
});
self.store.update(cx, |st, cx| {
    st.set_active_results(Vec::new(), next_gen, cx);
});
```
1. Clearing generates a normal, monotonically incrementing generation ID.
2. In-flight requests older than `next_gen` arriving after the clear are discarded.
3. Subsequent searches allocate `next_gen + 1`, successfully satisfying `generation >= self.in_flight_generation`.
