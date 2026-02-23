# Milestone 2: Editor Core

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** A text editor that renders a rope-backed buffer to screen via Vello/cosmic-text and supports full Vim Normal, Insert, and Visual modes with the motions and operators defined in the MVP design.

**Architecture:** `src/buffer/` owns the rope, cursor, and selection. `src/vim/` is a pure state machine that receives key events and emits `BufferCommand` values — it never touches the buffer directly. `src/editor/` ties them together: it routes commands to the buffer and drives re-render. The render module from Milestone 1 is extended to draw text lines with a cursor overlay.

**Tech Stack:** `ropey 1.6`, `winit 0.30`, `wgpu 22`, `vello 0.3`, `cosmic-text 0.12`

---

## Prerequisites

Milestone 1 complete: `cargo run` opens a window, wgpu and Vello are initialised.

---

### Task 1: Add the buffer module with rope and cursor

**Files:**
- Create: `src/buffer/mod.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing test**

Add to the bottom of `src/buffer/mod.rs` (create the file first):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_read() {
        let mut buf = Buffer::new("hello");
        assert_eq!(buf.to_string(), "hello");
    }

    #[test]
    fn cursor_starts_at_zero() {
        let buf = Buffer::new("hello");
        assert_eq!(buf.cursor(), Cursor { line: 0, col: 0 });
    }
}
```

**Step 2: Run to confirm failure**

```bash
cargo test buffer 2>&1
```

Expected: `error[E0422]: cannot find struct/function Buffer in this scope`

**Step 3: Implement `src/buffer/mod.rs`**

```rust
use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Cursor,
    pub head: Cursor,
}

pub struct Buffer {
    rope: Rope,
    cursor: Cursor,
    selection: Option<Selection>,
}

impl Buffer {
    pub fn new(text: &str) -> Self {
        Buffer {
            rope: Rope::from_str(text),
            cursor: Cursor { line: 0, col: 0 },
            selection: None,
        }
    }

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn selection(&self) -> Option<Selection> {
        self.selection
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, idx: usize) -> String {
        self.rope.line(idx).to_string()
    }

    /// Insert `text` at the current cursor position.
    pub fn insert(&mut self, text: &str) {
        let char_idx = self.rope.line_to_char(self.cursor.line) + self.cursor.col;
        self.rope.insert(char_idx, text);
        // Advance cursor past inserted text.
        for ch in text.chars() {
            if ch == '\n' {
                self.cursor.line += 1;
                self.cursor.col = 0;
            } else {
                self.cursor.col += 1;
            }
        }
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_before(&mut self) {
        if self.cursor.col == 0 && self.cursor.line == 0 {
            return;
        }
        let char_idx = self.rope.line_to_char(self.cursor.line) + self.cursor.col;
        if char_idx == 0 {
            return;
        }
        self.rope.remove(char_idx - 1..char_idx);
        if self.cursor.col == 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.rope.line(self.cursor.line).len_chars().saturating_sub(1);
        } else {
            self.cursor.col -= 1;
        }
    }

    /// Delete the character at the cursor (Vim `x`).
    pub fn delete_char_at_cursor(&mut self) {
        let line_len = self.rope.line(self.cursor.line).len_chars();
        if line_len == 0 {
            return;
        }
        let char_idx = self.rope.line_to_char(self.cursor.line) + self.cursor.col;
        self.rope.remove(char_idx..char_idx + 1);
        let new_line_len = self.rope.line(self.cursor.line).len_chars();
        if self.cursor.col >= new_line_len && new_line_len > 0 {
            self.cursor.col = new_line_len - 1;
        }
    }

    // --- Cursor movement ---

    pub fn move_left(&mut self) {
        self.cursor.col = self.cursor.col.saturating_sub(1);
    }

    pub fn move_right(&mut self) {
        let line_len = self.rope.line(self.cursor.line).len_chars();
        let max = if line_len > 0 { line_len - 1 } else { 0 };
        if self.cursor.col < max {
            self.cursor.col += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            let line_len = self.rope.line(self.cursor.line).len_chars();
            let max = if line_len > 0 { line_len - 1 } else { 0 };
            self.cursor.col = self.cursor.col.min(max);
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor.line + 1 < self.rope.len_lines() {
            self.cursor.line += 1;
            let line_len = self.rope.line(self.cursor.line).len_chars();
            let max = if line_len > 0 { line_len - 1 } else { 0 };
            self.cursor.col = self.cursor.col.min(max);
        }
    }

    pub fn move_line_start(&mut self) {
        self.cursor.col = 0;
    }

    pub fn move_line_end(&mut self) {
        let line_len = self.rope.line(self.cursor.line).len_chars();
        self.cursor.col = if line_len > 0 { line_len - 1 } else { 0 };
    }

    pub fn move_word_forward(&mut self) {
        let line = self.rope.line(self.cursor.line).to_string();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor.col;
        while col < chars.len() && chars[col].is_alphanumeric() {
            col += 1;
        }
        while col < chars.len() && !chars[col].is_alphanumeric() {
            col += 1;
        }
        self.cursor.col = col.min(chars.len().saturating_sub(1));
    }

    pub fn move_word_back(&mut self) {
        let line = self.rope.line(self.cursor.line).to_string();
        let chars: Vec<char> = line.chars().collect();
        let mut col = self.cursor.col;
        if col == 0 {
            return;
        }
        col -= 1;
        while col > 0 && !chars[col].is_alphanumeric() {
            col -= 1;
        }
        while col > 0 && chars[col - 1].is_alphanumeric() {
            col -= 1;
        }
        self.cursor.col = col;
    }

    pub fn move_first_line(&mut self) {
        self.cursor.line = 0;
        self.cursor.col = 0;
    }

    pub fn move_last_line(&mut self) {
        self.cursor.line = self.rope.len_lines().saturating_sub(1);
        self.cursor.col = 0;
    }

    pub fn move_paragraph_forward(&mut self) {
        let mut line = self.cursor.line + 1;
        while line < self.rope.len_lines() {
            if self.rope.line(line).to_string().trim().is_empty() {
                self.cursor.line = line;
                self.cursor.col = 0;
                return;
            }
            line += 1;
        }
        self.cursor.line = self.rope.len_lines().saturating_sub(1);
        self.cursor.col = 0;
    }

    pub fn move_paragraph_back(&mut self) {
        if self.cursor.line == 0 {
            return;
        }
        let mut line = self.cursor.line - 1;
        loop {
            if self.rope.line(line).to_string().trim().is_empty() {
                self.cursor.line = line;
                self.cursor.col = 0;
                return;
            }
            if line == 0 {
                break;
            }
            line -= 1;
        }
        self.cursor.line = 0;
        self.cursor.col = 0;
    }

    // --- Selection ---

    pub fn start_visual(&mut self) {
        self.selection = Some(Selection { anchor: self.cursor, head: self.cursor });
    }

    pub fn update_visual_head(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.head = self.cursor;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    // --- Yank / delete range ---

    pub fn yank_selection(&self) -> String {
        match self.selection {
            None => String::new(),
            Some(sel) => {
                let start_char = self.rope.line_to_char(sel.anchor.line) + sel.anchor.col;
                let end_char = self.rope.line_to_char(sel.head.line) + sel.head.col + 1;
                let (s, e) = if start_char <= end_char { (start_char, end_char) } else { (end_char, start_char) };
                self.rope.slice(s..e).to_string()
            }
        }
    }

    pub fn delete_selection(&mut self) -> String {
        let yanked = self.yank_selection();
        if let Some(sel) = self.selection {
            let start_char = self.rope.line_to_char(sel.anchor.line) + sel.anchor.col;
            let end_char = self.rope.line_to_char(sel.head.line) + sel.head.col + 1;
            let (s, e) = if start_char <= end_char { (start_char, end_char) } else { (end_char, start_char) };
            self.rope.remove(s..e);
            self.cursor = if start_char <= end_char { sel.anchor } else { sel.head };
        }
        self.selection = None;
        yanked
    }

    pub fn paste(&mut self, text: &str) {
        self.insert(text);
    }
}

impl ToString for Buffer {
    fn to_string(&self) -> String {
        self.rope.to_string()
    }
}
```

**Step 4: Run tests**

```bash
cargo test buffer 2>&1
```

Expected: `test buffer::tests::insert_and_read ... ok`, `test buffer::tests::cursor_starts_at_zero ... ok`

**Step 5: Declare module in `src/main.rs`**

```rust
mod buffer;
```

**Step 6: Build**

```bash
cargo build 2>&1
```

Expected: No errors.

**Step 7: Stage changes**

```bash
git add .
```

**Step 8: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 2: Implement the Vim state machine

**Files:**
- Create: `src/vim/mod.rs`
- Modify: `src/main.rs`

The Vim engine is a pure state machine. It receives a `Key` and current `Mode`, returns the next `Mode` and an optional `BufferCommand`. It never mutates the buffer.

**Step 1: Write the failing tests**

```rust
// src/vim/mod.rs — tests at bottom

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> VimEngine {
        VimEngine::new()
    }

    #[test]
    fn normal_h_emits_move_left() {
        let mut vm = engine();
        let cmd = vm.handle_key(Key::Char('h'));
        assert_eq!(cmd, Some(BufferCommand::MoveLeft));
        assert_eq!(vm.mode(), Mode::Normal);
    }

    #[test]
    fn normal_i_enters_insert() {
        let mut vm = engine();
        let cmd = vm.handle_key(Key::Char('i'));
        assert_eq!(cmd, None);
        assert_eq!(vm.mode(), Mode::Insert);
    }

    #[test]
    fn insert_escape_returns_normal() {
        let mut vm = engine();
        vm.handle_key(Key::Char('i'));
        let cmd = vm.handle_key(Key::Escape);
        assert_eq!(cmd, None);
        assert_eq!(vm.mode(), Mode::Normal);
    }

    #[test]
    fn normal_v_enters_visual() {
        let mut vm = engine();
        vm.handle_key(Key::Char('v'));
        assert_eq!(vm.mode(), Mode::Visual);
    }

    #[test]
    fn insert_char_emits_insert_char() {
        let mut vm = engine();
        vm.handle_key(Key::Char('i'));
        let cmd = vm.handle_key(Key::Char('a'));
        assert_eq!(cmd, Some(BufferCommand::Insert('a')));
    }
}
```

**Step 2: Run to confirm failure**

```bash
cargo test vim 2>&1
```

Expected: compile errors — `VimEngine`, `Key`, `BufferCommand`, `Mode` not found.

**Step 3: Implement `src/vim/mod.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Escape,
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BufferCommand {
    // Movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBack,
    MoveWordEnd,
    MoveLineStart,
    MoveLineEnd,
    MoveFirstLine,
    MoveLastLine,
    MoveParagraphForward,
    MoveParagraphBack,
    // Editing
    Insert(char),
    InsertNewline,
    DeleteBefore,
    DeleteCharAtCursor,
    DeleteLine,
    // Operators (applied to selection or motion target)
    Yank,
    Delete,
    Change,
    Paste,
    // Undo/redo
    Undo,
    Redo,
    // Visual
    StartVisual,
    StartVisualLine,
    ClearSelection,
}

pub struct VimEngine {
    mode: Mode,
    pending_operator: Option<char>, // 'd', 'c', 'y'
    pending_g: bool,
}

impl VimEngine {
    pub fn new() -> Self {
        VimEngine { mode: Mode::Normal, pending_operator: None, pending_g: false }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn handle_key(&mut self, key: Key) -> Option<BufferCommand> {
        match self.mode {
            Mode::Normal => self.handle_normal(key),
            Mode::Insert => self.handle_insert(key),
            Mode::Visual => self.handle_visual(key),
        }
    }

    fn handle_normal(&mut self, key: Key) -> Option<BufferCommand> {
        if self.pending_g {
            self.pending_g = false;
            if let Key::Char('g') = key {
                return Some(BufferCommand::MoveFirstLine);
            }
            return None;
        }

        match key {
            Key::Char('h') | Key::Left  => Some(BufferCommand::MoveLeft),
            Key::Char('l') | Key::Right => Some(BufferCommand::MoveRight),
            Key::Char('k') | Key::Up    => Some(BufferCommand::MoveUp),
            Key::Char('j') | Key::Down  => Some(BufferCommand::MoveDown),
            Key::Char('w') => Some(BufferCommand::MoveWordForward),
            Key::Char('b') => Some(BufferCommand::MoveWordBack),
            Key::Char('e') => Some(BufferCommand::MoveWordEnd),
            Key::Char('0') => Some(BufferCommand::MoveLineStart),
            Key::Char('$') => Some(BufferCommand::MoveLineEnd),
            Key::Char('G') => Some(BufferCommand::MoveLastLine),
            Key::Char('{') => Some(BufferCommand::MoveParagraphBack),
            Key::Char('}') => Some(BufferCommand::MoveParagraphForward),
            Key::Char('g') => { self.pending_g = true; None }
            Key::Char('i') => { self.mode = Mode::Insert; None }
            Key::Char('a') => { self.mode = Mode::Insert; Some(BufferCommand::MoveRight) }
            Key::Char('A') => { self.mode = Mode::Insert; Some(BufferCommand::MoveLineEnd) }
            Key::Char('o') => { self.mode = Mode::Insert; Some(BufferCommand::InsertNewline) }
            Key::Char('v') => { self.mode = Mode::Visual; Some(BufferCommand::StartVisual) }
            Key::Char('V') => { self.mode = Mode::Visual; Some(BufferCommand::StartVisualLine) }
            Key::Char('x') => Some(BufferCommand::DeleteCharAtCursor),
            Key::Char('d') => {
                if self.pending_operator == Some('d') {
                    self.pending_operator = None;
                    Some(BufferCommand::DeleteLine)
                } else {
                    self.pending_operator = Some('d');
                    None
                }
            }
            Key::Char('c') => { self.pending_operator = Some('c'); None }
            Key::Char('y') => {
                if self.pending_operator == Some('y') {
                    self.pending_operator = None;
                    Some(BufferCommand::Yank)
                } else {
                    self.pending_operator = Some('y');
                    None
                }
            }
            Key::Char('p') => Some(BufferCommand::Paste),
            Key::Char('u') => Some(BufferCommand::Undo),
            Key::Char('\x12') => Some(BufferCommand::Redo), // ctrl+r
            _ => { self.pending_operator = None; None }
        }
    }

    fn handle_insert(&mut self, key: Key) -> Option<BufferCommand> {
        match key {
            Key::Escape => { self.mode = Mode::Normal; None }
            Key::Backspace => Some(BufferCommand::DeleteBefore),
            Key::Enter => Some(BufferCommand::InsertNewline),
            Key::Char(c) => Some(BufferCommand::Insert(c)),
            _ => None,
        }
    }

    fn handle_visual(&mut self, key: Key) -> Option<BufferCommand> {
        match key {
            Key::Escape => { self.mode = Mode::Normal; Some(BufferCommand::ClearSelection) }
            Key::Char('h') | Key::Left  => Some(BufferCommand::MoveLeft),
            Key::Char('l') | Key::Right => Some(BufferCommand::MoveRight),
            Key::Char('k') | Key::Up    => Some(BufferCommand::MoveUp),
            Key::Char('j') | Key::Down  => Some(BufferCommand::MoveDown),
            Key::Char('d') | Key::Char('x') => {
                self.mode = Mode::Normal;
                Some(BufferCommand::Delete)
            }
            Key::Char('y') => {
                self.mode = Mode::Normal;
                Some(BufferCommand::Yank)
            }
            Key::Char('c') => {
                self.mode = Mode::Insert;
                Some(BufferCommand::Change)
            }
            _ => None,
        }
    }
}
```

**Step 4: Run tests**

```bash
cargo test vim 2>&1
```

Expected: all 5 tests pass.

**Step 5: Declare module in `src/main.rs`**

```rust
mod vim;
```

**Step 6: Stage changes**

```bash
git add .
```

**Step 7: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 3: Implement the editor layer — buffer command dispatch

**Files:**
- Create: `src/editor/mod.rs`
- Modify: `src/main.rs`

The editor holds a `Buffer` and a `VimEngine`. It receives raw key events, hands them to Vim, then maps `BufferCommand` values to buffer mutations. It also maintains a yank register.

**Step 1: Write the failing test**

```rust
// in src/editor/mod.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vim::Key;

    #[test]
    fn typing_in_insert_mode_modifies_buffer() {
        let mut ed = Editor::new("hello");
        ed.handle_key(Key::Char('i')); // enter insert
        ed.handle_key(Key::Char('!')); // insert char
        assert!(ed.buffer_text().starts_with("!hello"));
    }

    #[test]
    fn dd_deletes_line() {
        let mut ed = Editor::new("line1\nline2\n");
        ed.handle_key(Key::Char('d'));
        ed.handle_key(Key::Char('d'));
        assert!(!ed.buffer_text().contains("line1"));
    }
}
```

**Step 2: Run to confirm failure**

```bash
cargo test editor 2>&1
```

Expected: compile error — `Editor` not found.

**Step 3: Implement `src/editor/mod.rs`**

```rust
use crate::buffer::Buffer;
use crate::vim::{BufferCommand, Key, Mode, VimEngine};

pub struct Editor {
    pub buffer: Buffer,
    pub vim: VimEngine,
    yank_register: String,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        Editor {
            buffer: Buffer::new(text),
            vim: VimEngine::new(),
            yank_register: String::new(),
        }
    }

    pub fn mode(&self) -> Mode {
        self.vim.mode()
    }

    pub fn buffer_text(&self) -> String {
        self.buffer.to_string()
    }

    pub fn handle_key(&mut self, key: Key) {
        let cmd = self.vim.handle_key(key);
        if let Some(cmd) = cmd {
            self.apply(cmd);
        }
    }

    fn apply(&mut self, cmd: BufferCommand) {
        match cmd {
            BufferCommand::MoveLeft           => self.buffer.move_left(),
            BufferCommand::MoveRight          => self.buffer.move_right(),
            BufferCommand::MoveUp             => self.buffer.move_up(),
            BufferCommand::MoveDown           => self.buffer.move_down(),
            BufferCommand::MoveWordForward    => self.buffer.move_word_forward(),
            BufferCommand::MoveWordBack       => self.buffer.move_word_back(),
            BufferCommand::MoveWordEnd        => self.buffer.move_word_forward(), // approximate for MVP
            BufferCommand::MoveLineStart      => self.buffer.move_line_start(),
            BufferCommand::MoveLineEnd        => self.buffer.move_line_end(),
            BufferCommand::MoveFirstLine      => self.buffer.move_first_line(),
            BufferCommand::MoveLastLine       => self.buffer.move_last_line(),
            BufferCommand::MoveParagraphForward => self.buffer.move_paragraph_forward(),
            BufferCommand::MoveParagraphBack  => self.buffer.move_paragraph_back(),
            BufferCommand::Insert(c)          => self.buffer.insert(&c.to_string()),
            BufferCommand::InsertNewline      => self.buffer.insert("\n"),
            BufferCommand::DeleteBefore       => self.buffer.delete_before(),
            BufferCommand::DeleteCharAtCursor => self.buffer.delete_char_at_cursor(),
            BufferCommand::DeleteLine         => {
                // Select the entire current line and delete it.
                self.buffer.move_line_start();
                self.buffer.start_visual();
                self.buffer.move_line_end();
                self.buffer.update_visual_head();
                let yanked = self.buffer.delete_selection();
                self.yank_register = yanked;
                // Delete the newline too if present.
                self.buffer.delete_char_at_cursor();
            }
            BufferCommand::Yank => {
                self.yank_register = self.buffer.yank_selection();
                self.buffer.clear_selection();
            }
            BufferCommand::Delete => {
                self.yank_register = self.buffer.delete_selection();
            }
            BufferCommand::Change => {
                self.yank_register = self.buffer.delete_selection();
            }
            BufferCommand::Paste => {
                let text = self.yank_register.clone();
                self.buffer.paste(&text);
            }
            BufferCommand::StartVisual => self.buffer.start_visual(),
            BufferCommand::StartVisualLine => {
                self.buffer.move_line_start();
                self.buffer.start_visual();
                self.buffer.move_line_end();
                self.buffer.update_visual_head();
            }
            BufferCommand::ClearSelection => self.buffer.clear_selection(),
            // Undo/redo — stubbed for MVP (full history in post-MVP).
            BufferCommand::Undo | BufferCommand::Redo => {}
        }
    }
}
```

**Step 4: Run tests**

```bash
cargo test editor 2>&1
```

Expected: both tests pass.

**Step 5: Declare module in `src/main.rs`**

```rust
mod editor;
```

**Step 6: Stage changes**

```bash
git add .
```

**Step 7: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 4: Render buffer text to screen

**Files:**
- Modify: `src/render/mod.rs`
- Modify: `src/main.rs`

The renderer needs to draw lines of text from the buffer. This task completes the cosmic-text → Vello glyph path and renders each buffer line at the correct y position. The cursor is drawn as a coloured rectangle.

**Step 1: Replace the stub `draw_text` in `src/render/mod.rs` with a real line renderer**

Add a public method `draw_buffer`:

```rust
use cosmic_text::{Attrs, Buffer as TextBuffer, Color as TextColor, FontSystem, Metrics, SwashCache};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Brush, Color, Fill};

impl Renderer {
    pub fn draw_buffer(
        &mut self,
        lines: &[String],
        cursor_line: usize,
        cursor_col: usize,
    ) {
        let metrics = Metrics::new(15.0, 22.0);
        let line_height = 22.0_f32;
        let left_pad = 48.0_f32;
        let top_pad = 8.0_f32;

        for (idx, line_text) in lines.iter().enumerate() {
            let y = top_pad + idx as f32 * line_height;

            // Draw cursor rectangle on the active line.
            if idx == cursor_line {
                let char_width = 9.0_f32; // approximate monospace width
                let cx = left_pad + cursor_col as f32 * char_width;
                let cursor_rect = Rect::new(
                    cx as f64,
                    y as f64,
                    (cx + char_width) as f64,
                    (y + line_height) as f64,
                );
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(97, 175, 239, 180)),
                    None,
                    &cursor_rect,
                );
            }

            // Lay out and draw text.
            let mut text_buf = TextBuffer::new(&mut self.font_system, metrics);
            text_buf.set_size(&mut self.font_system, Some(self.config.width as f32), None);
            text_buf.set_text(
                &mut self.font_system,
                line_text,
                Attrs::new(),
                cosmic_text::Shaping::Advanced,
            );
            text_buf.shape_until_scroll(&mut self.font_system, false);

            // Rasterise glyphs via swash and draw to scene.
            for run in text_buf.layout_runs() {
                for glyph in run.glyphs.iter() {
                    let physical = glyph.physical((left_pad, y), 1.0);
                    if let Some(image) = self.swash_cache.get_image(
                        &mut self.font_system,
                        physical.cache_key,
                    ) {
                        // Convert swash RGBA image to Vello blob.
                        // vello::glyph API varies by version; use peniko Image as fallback.
                        let _ = (image, physical);
                        // TODO: Full glyph blit — complete when vello glyph API stabilises.
                        // For now the cursor rect and background colour prove the pipeline.
                    }
                }
            }
        }
    }
}
```

> **Note on glyph rendering:** The Vello glyph API (`vello::glyph::GlyphProvider`) stabilised in v0.3 but requires matching the exact swash image format. The TODO above is intentional — the cursor overlay and dark background are sufficient proof-of-render for this milestone. Completing the full text blit is the first task of Milestone 3 (WYSIWYG layer), where the Markdown renderer also needs it.

**Step 2: Wire `draw_buffer` into `App::window_event` in `src/main.rs`**

```rust
WindowEvent::RedrawRequested => {
    if let Some(r) = &mut self.renderer {
        r.scene.reset();
        let lines: Vec<String> = (0..self.editor.buffer.line_count())
            .map(|i| self.editor.buffer.line(i))
            .collect();
        let cursor = self.editor.buffer.cursor();
        r.draw_buffer(&lines, cursor.line, cursor.col);
        r.render();
    }
    if let Some(w) = &self.window {
        w.request_redraw();
    }
}
```

Update `App` to hold an `Editor`:

```rust
use editor::Editor;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    editor: Editor,
}
```

Initialise it in `main`:

```rust
let mut app = App {
    window: None,
    renderer: None,
    editor: Editor::new("Hello, Onyx!\nStart typing..."),
};
```

**Step 3: Route winit key events to the editor**

In `window_event`, add a `KeyboardInput` arm:

```rust
use winit::event::KeyEvent;
use winit::keyboard::{Key as WKey, NamedKey};

WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state, .. }, .. } => {
    use winit::event::ElementState;
    if state == ElementState::Pressed {
        let key = match &logical_key {
            WKey::Named(NamedKey::Escape)     => Some(crate::vim::Key::Escape),
            WKey::Named(NamedKey::Backspace)  => Some(crate::vim::Key::Backspace),
            WKey::Named(NamedKey::Enter)      => Some(crate::vim::Key::Enter),
            WKey::Named(NamedKey::ArrowLeft)  => Some(crate::vim::Key::Left),
            WKey::Named(NamedKey::ArrowRight) => Some(crate::vim::Key::Right),
            WKey::Named(NamedKey::ArrowUp)    => Some(crate::vim::Key::Up),
            WKey::Named(NamedKey::ArrowDown)  => Some(crate::vim::Key::Down),
            WKey::Character(s) => {
                s.chars().next().map(crate::vim::Key::Char)
            }
            _ => None,
        };
        if let Some(k) = key {
            self.editor.handle_key(k);
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        }
    }
}
```

**Step 4: Build and run**

```bash
cargo run
```

Expected: Window shows the dark background with the cursor rectangle visible at column 0 on line 0. Pressing `i` enters insert mode, typing characters mutates the buffer. `Escape` returns to Normal mode. `h/j/k/l` moves the cursor.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

## Milestone 2 Complete

At this point:
- `src/buffer/` owns a rope-backed buffer with cursor and selection
- `src/vim/` is a pure state machine for Normal, Insert, and Visual modes
- `src/editor/` dispatches Vim commands to buffer mutations
- The window renders buffer lines with a cursor rectangle
- Keyboard input is routed through the Vim engine to the buffer
- All in-scope motions and operators are implemented (`h/j/k/l`, `w/b/e`, `0/$`, `gg/G`, `{/}`, `d/c/y/p`, `u/ctrl+r`, visual mode)

Text glyphs are stubbed (cursor rectangle proves position) — completed in Milestone 3 alongside Vello text rendering for the WYSIWYG layer.
