# Plan: Make the Editor Editable

## Context

Onyx is a GPU-rendered markdown editor (winit + vello + cosmic-text). The file tree, tab bar, and read-only text display are complete. The editor currently stores file content as `Vec<String>` in each `Tab` and has zero editing capability — no cursor, no text input, no save, no scrolling. This plan adds the minimal editing stack so users can open a markdown file, type into it, and save.

## Milestones

### 1. Buffer Abstraction

**Create** `src/buffer.rs` — rope-backed buffer using `ropey` (add to Cargo.toml).

```rust
pub struct Buffer {
    rope: Rope,
    cursor: CursorPosition,  // { row, col }
    goal_col: Option<usize>, // sticky column for vertical movement
    dirty: bool,
    path: PathBuf,
}
```

Key methods: `from_file`, `from_string`, `line(index)`, `line_count`, `cursor`, `is_dirty`, `insert_char`, `insert_newline`, `delete_backward`, `delete_forward`, `move_cursor_{up,down,left,right,home,end}`, `set_cursor`, `save`.

**Modify** `src/editor_view.rs` — replace `Tab { path, name, content_lines }` with `Tab { name, buffer: Buffer }`. Update render loop to iterate `buffer.line(row)`.

**Modify** `src/main.rs` — add `mod buffer`.

Tests: round-trip content, insert/delete at edges, newline splits/joins, cursor clamping, goal_col preservation, dirty flag lifecycle, save writes to disk.

---

### 2. Cursor Rendering

Draw a 2px-wide vertical caret at the cursor position using `measure_text` for accurate x-offset.

**Modify** `src/editor_view.rs` — after text rendering, measure prefix text up to `cursor.col`, draw a `Panel` rect at that position when visible.

**Modify** `src/app.rs` — add `cursor_blink_visible: bool` and `last_blink_toggle: Instant` to `App`. Toggle every 530ms. Reset to visible on any keypress.

**Modify** `src/ui/canvas.rs` — add `cursor_visible: bool` to `DrawContext`.

Tests: prefix measurement returns 0 for col 0, matches `measure_text` for known strings. Blink toggle flips after 530ms.

---

### 3. Text Input

Forward winit keyboard events to buffer mutation methods.

**Modify** `src/app.rs` — expand `KeyboardInput` handler for `Editor` screen: `Backspace` → `delete_backward`, `Delete` → `delete_forward`, `Enter` → `insert_newline`, `Key::Character` with `event.text` → `insert_char` (filter control chars).

**Modify** `src/editor_view.rs` — add `handle_char_input`, `handle_key_backspace`, `handle_key_delete`, `handle_key_enter`, `active_tab_mut()`.

Tests: char input modifies buffer, backspace/delete/enter produce correct content, no-op with no active tab.

---

### 4. Cursor Movement

Arrow keys, Home/End, click-to-position.

**Modify** `src/app.rs` — handle `ArrowUp/Down/Left/Right`, `Home`, `End`.

**Modify** `src/editor_view.rs` — add `handle_cursor_move(direction)`, `handle_content_click(position, text_system, theme)`. Store `content_rect: Option<Rect>` (set during render) for click-to-row/col conversion. Use binary search over `measure_text` calls to find the column from a pixel x-coordinate.

**Modify** `src/app.rs` — in `MouseInput`, if no hit in content area, call `handle_content_click`.

Tests: movement clamping, home/end, `col_from_pixel_x` with known strings.

---

### 5. Save (Cmd+S)

**Modify** `src/app.rs` — track `modifiers: winit::event::Modifiers` via `ModifiersChanged`. When super key is held and `s` is pressed, call `editor.handle_save()` instead of inserting `s`.

**Modify** `src/editor_view.rs` — add `handle_save()` that calls `buffer.save()` with error logging.

`Buffer::save` writes `rope.to_string()` via `std::fs::write`, then clears dirty flag.

Tests: save writes correct content to disk, clears dirty, returns error on bad path.

---

### 6. Dirty Indicator

**Modify** `src/editor_view.rs` — in tab bar rendering, prepend `"\u{2022} "` (bullet) to the tab label when `tab.buffer.is_dirty()`.

Tests: label formatting helper produces correct output for dirty/clean states.

---

### 7. Scrolling

**Modify** `src/editor_view.rs` — add `scroll_offset: f32` to `Tab`. Render only visible lines (`first_visible_line..last_visible_line`). Store `visible_height` from content rect each frame. Add `handle_scroll(delta)` with clamping and `ensure_cursor_visible()` called after every edit/move.

**Modify** `src/app.rs` — handle `MouseWheel` events, forward pixel delta to `editor.handle_scroll()`.

Adjust click-to-position (Milestone 4) to account for scroll offset.

Tests: scroll clamping at 0 and max, ensure_cursor_visible adjusts offset, visible line range computation.

## Files Modified

| File | Milestones |
|------|-----------|
| `Cargo.toml` | 1 |
| `src/main.rs` | 1 |
| `src/buffer.rs` (new) | 1 |
| `src/editor_view.rs` | 1–7 |
| `src/app.rs` | 2–5, 7 |
| `src/ui/canvas.rs` | 2 |

## Verification

After each milestone:
1. `cargo test` — all tests pass
2. `make format && make lint` — no warnings
3. Manual check — open a vault, open a markdown file, verify the milestone's behavior works
4. After all 7: open file → type text → arrow around → scroll → Cmd+S → reopen → content persisted
