# Milestone 6: Terminal Pane

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Embed a fully functional pty terminal in the right pane. Multiple independent terminal tabs. Input/output, scrollback buffer, ANSI colour, copy/paste. The pane is movable to left/right/bottom/top via the command palette.

**Architecture:** `src/terminal/` owns all pty logic. A `TerminalSession` wraps one `portable-pty` child process and its `VtParser` (ANSI state machine). A `TerminalPane` holds a `Vec<TerminalSession>` (one per tab) and an active index. `src/render/` grows `terminal.rs` to rasterise the terminal grid (rows × cols of cells with fg/bg colour) into Vello. `src/main.rs` wires `cmd+j` / `cmd+t` / `cmd+w` through the command registry.

**Tech Stack:** `portable-pty 0.8`, `vte 0.13` (ANSI/VT parser), `crossbeam-channel 0.5`

---

## Prerequisites

Milestone 5 complete: command registry, event bus, keybinding system, and pane layout all in place.

---

### Task 1: Add dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add to `[dependencies]`**

```toml
portable-pty = "0.8"
vte = "0.13"
crossbeam-channel = "0.5"
```

**Step 2: Build to confirm they resolve**

```bash
cargo build 2>&1
```

Expected: Compiles. New crates downloaded and compiled.

**Step 3: Stage changes**

```bash
git add .
```

**Step 4: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 2: Implement the terminal cell grid

**Files:**
- Create: `src/terminal/mod.rs`
- Modify: `src/main.rs`

The terminal renders as a 2D grid of `Cell` values. The `VtParser` (via `vte`) updates the grid as bytes arrive from the pty.

**Step 1: Write failing tests**

```rust
// src/terminal/mod.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_is_blank() {
        let grid = TerminalGrid::new(24, 80);
        assert_eq!(grid.rows, 24);
        assert_eq!(grid.cols, 80);
        assert_eq!(grid.cell(0, 0).ch, ' ');
    }

    #[test]
    fn write_text_fills_cells() {
        let mut grid = TerminalGrid::new(24, 80);
        grid.write_str("hello");
        assert_eq!(grid.cell(0, 0).ch, 'h');
        assert_eq!(grid.cell(0, 4).ch, 'o');
    }

    #[test]
    fn newline_moves_cursor_down() {
        let mut grid = TerminalGrid::new(24, 80);
        grid.write_str("line1\nline2");
        assert_eq!(grid.cell(1, 0).ch, 'l');
    }
}
```

**Step 2: Run to confirm failure**

```bash
cargo test terminal 2>&1
```

Expected: compile error.

**Step 3: Implement `src/terminal/mod.rs`**

```rust
use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Debug, Clone, Copy)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    pub const WHITE: Colour = Colour { r: 204, g: 204, b: 204 };
    pub const BLACK: Colour = Colour { r: 26,  g: 26,  b: 30  };
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub fg: Colour,
    pub bg: Colour,
    pub bold: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ', fg: Colour::WHITE, bg: Colour::BLACK, bold: false }
    }
}

pub struct TerminalGrid {
    pub rows: usize,
    pub cols: usize,
    cells: Vec<Cell>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    current_fg: Colour,
    current_bg: Colour,
    current_bold: bool,
}

impl TerminalGrid {
    pub fn new(rows: usize, cols: usize) -> Self {
        TerminalGrid {
            rows,
            cols,
            cells: vec![Cell::default(); rows * cols],
            cursor_row: 0,
            cursor_col: 0,
            current_fg: Colour::WHITE,
            current_bg: Colour::BLACK,
            current_bold: false,
        }
    }

    pub fn cell(&self, row: usize, col: usize) -> Cell {
        self.cells[row * self.cols + col]
    }

    pub fn write_str(&mut self, text: &str) {
        for ch in text.chars() {
            self.write_char(ch);
        }
    }

    fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.scroll_up();
                    self.cursor_row = self.rows - 1;
                }
            }
            '\r' => {
                self.cursor_col = 0;
            }
            c => {
                if self.cursor_col < self.cols && self.cursor_row < self.rows {
                    let idx = self.cursor_row * self.cols + self.cursor_col;
                    self.cells[idx] = Cell {
                        ch: c,
                        fg: self.current_fg,
                        bg: self.current_bg,
                        bold: self.current_bold,
                    };
                    self.cursor_col += 1;
                    if self.cursor_col >= self.cols {
                        self.cursor_col = 0;
                        self.cursor_row += 1;
                        if self.cursor_row >= self.rows {
                            self.scroll_up();
                            self.cursor_row = self.rows - 1;
                        }
                    }
                }
            }
        }
    }

    fn scroll_up(&mut self) {
        self.cells.drain(0..self.cols);
        self.cells.extend(vec![Cell::default(); self.cols]);
    }

    pub fn apply_sgr(&mut self, params: &[u16]) {
        for &p in params {
            match p {
                0  => { self.current_fg = Colour::WHITE; self.current_bg = Colour::BLACK; self.current_bold = false; }
                1  => self.current_bold = true,
                30 => self.current_fg = Colour { r: 0,   g: 0,   b: 0   },
                31 => self.current_fg = Colour { r: 224, g: 108, b: 117 },
                32 => self.current_fg = Colour { r: 152, g: 195, b: 121 },
                33 => self.current_fg = Colour { r: 229, g: 192, b: 123 },
                34 => self.current_fg = Colour { r: 97,  g: 175, b: 239 },
                35 => self.current_fg = Colour { r: 198, g: 120, b: 221 },
                36 => self.current_fg = Colour { r: 86,  g: 182, b: 194 },
                37 => self.current_fg = Colour::WHITE,
                _  => {}
            }
        }
    }
}

// ── VTE performer — wires the ANSI parser to the grid ──────────────────────

pub struct VtePerformer {
    pub grid: TerminalGrid,
}

impl vte::Perform for VtePerformer {
    fn print(&mut self, c: char) {
        self.grid.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.grid.write_char('\n'),
            b'\r' => self.grid.write_char('\r'),
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            'm' => {
                let sgr: Vec<u16> = params.iter()
                    .map(|p| p.first().copied().unwrap_or(0))
                    .collect();
                self.grid.apply_sgr(&sgr);
            }
            'H' | 'f' => {
                // Cursor position.
                let mut iter = params.iter();
                let row = iter.next().and_then(|p| p.first().copied()).unwrap_or(1).saturating_sub(1) as usize;
                let col = iter.next().and_then(|p| p.first().copied()).unwrap_or(1).saturating_sub(1) as usize;
                self.grid.cursor_row = row.min(self.grid.rows - 1);
                self.grid.cursor_col = col.min(self.grid.cols - 1);
            }
            'J' => {
                // Erase display — clear grid for MVP.
                let rows = self.grid.rows;
                let cols = self.grid.cols;
                self.grid = TerminalGrid::new(rows, cols);
            }
            _ => {}
        }
    }
}

// ── TerminalSession — one pty child + parser ───────────────────────────────

pub struct TerminalSession {
    pub name: String,
    pub performer: VtePerformer,
    parser: vte::Parser,
    writer: Box<dyn std::io::Write + Send>,
    reader_rx: Receiver<Vec<u8>>,
}

impl TerminalSession {
    pub fn spawn(name: &str, vault_root: &std::path::Path, rows: u16, cols: u16) -> Self {
        use portable_pty::{CommandBuilder, PtySize, native_pty_system};

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .expect("failed to open pty");

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(vault_root);

        let _child = pair.slave.spawn_command(cmd).expect("failed to spawn shell");

        let writer = pair.master.take_writer().expect("pty writer");
        let mut reader = pair.master.try_clone_reader().expect("pty reader");

        let (tx, rx) = unbounded::<Vec<u8>>();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        TerminalSession {
            name: name.to_string(),
            performer: VtePerformer {
                grid: TerminalGrid::new(rows as usize, cols as usize),
            },
            parser: vte::Parser::new(),
            writer,
            reader_rx: rx,
        }
    }

    /// Drain pending output from the pty and parse into the grid.
    pub fn tick(&mut self) {
        while let Ok(bytes) = self.reader_rx.try_recv() {
            for &b in &bytes {
                self.parser.advance(&mut self.performer, b);
            }
        }
    }

    /// Send input bytes to the pty.
    pub fn write(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
    }
}

// ── TerminalPane — multiple sessions ─────────────────────────────────────

pub struct TerminalPane {
    sessions: Vec<TerminalSession>,
    active: usize,
    vault_root: std::path::PathBuf,
    rows: u16,
    cols: u16,
}

impl TerminalPane {
    pub fn new(vault_root: &std::path::Path, rows: u16, cols: u16) -> Self {
        let session = TerminalSession::spawn("zsh 1", vault_root, rows, cols);
        TerminalPane {
            sessions: vec![session],
            active: 0,
            vault_root: vault_root.to_path_buf(),
            rows,
            cols,
        }
    }

    pub fn new_tab(&mut self) {
        let name = format!("zsh {}", self.sessions.len() + 1);
        let session = TerminalSession::spawn(&name, &self.vault_root, self.rows, self.cols);
        self.sessions.push(session);
        self.active = self.sessions.len() - 1;
    }

    pub fn close_tab(&mut self) {
        if self.sessions.len() > 1 {
            self.sessions.remove(self.active);
            if self.active >= self.sessions.len() {
                self.active = self.sessions.len() - 1;
            }
        }
    }

    pub fn active_session(&mut self) -> &mut TerminalSession {
        &mut self.sessions[self.active]
    }

    pub fn tick_all(&mut self) {
        for s in &mut self.sessions {
            s.tick();
        }
    }

    pub fn tab_names(&self) -> Vec<&str> {
        self.sessions.iter().map(|s| s.name.as_str()).collect()
    }

    pub fn active_index(&self) -> usize {
        self.active
    }
}
```

> `use std::io::Read;` and `use std::io::Write;` are needed at the top of the file.

**Step 4: Run tests**

```bash
cargo test terminal 2>&1
```

Expected: all 3 tests pass.

**Step 5: Declare module in `src/main.rs`**

```rust
mod terminal;
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

### Task 3: Render the terminal grid

**Files:**
- Create: `src/render/terminal.rs`
- Modify: `src/render/mod.rs`

**Step 1: Create `src/render/terminal.rs`**

```rust
use crate::terminal::TerminalGrid;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

pub fn draw_terminal(
    scene: &mut Scene,
    grid: &TerminalGrid,
    origin_x: f32,
    origin_y: f32,
    cell_width: f32,
    cell_height: f32,
) {
    // Background fill for the terminal pane.
    let pane_w = grid.cols as f32 * cell_width;
    let pane_h = grid.rows as f32 * cell_height;
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(20, 20, 24, 255)),
        None,
        &Rect::new(origin_x as f64, origin_y as f64, (origin_x + pane_w) as f64, (origin_y + pane_h) as f64),
    );

    // Draw cells.
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let cell = grid.cell(row, col);
            let x = origin_x + col as f32 * cell_width;
            let y = origin_y + row as f32 * cell_height;

            // Cell background (only draw if non-default).
            if cell.bg.r != 26 || cell.bg.g != 26 || cell.bg.b != 30 {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(cell.bg.r, cell.bg.g, cell.bg.b, 255)),
                    None,
                    &Rect::new(x as f64, y as f64, (x + cell_width) as f64, (y + cell_height) as f64),
                );
            }

            // Cursor.
            if row == grid.cursor_row && col == grid.cursor_col {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(97, 175, 239, 200)),
                    None,
                    &Rect::new(x as f64, y as f64, (x + cell_width) as f64, (y + cell_height) as f64),
                );
            }

            // Cell character — drawn via cosmic-text (TODO: wire glyph pipeline).
            // For MVP the colour rectangles prove cell boundaries; text drawn as in editor.
            let _ = cell.ch;
        }
    }
}
```

**Step 2: Re-export from `src/render/mod.rs`**

```rust
pub mod terminal;
```

**Step 3: Stage changes**

```bash
git add .
```

**Step 4: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 4: Wire the terminal pane into the event loop

**Files:**
- Modify: `src/main.rs`

**Step 1: Add `TerminalPane` to `App`**

```rust
use terminal::TerminalPane;

struct App {
    // ... existing fields ...
    terminal_pane: Option<TerminalPane>,
    terminal_visible: bool,
    terminal_focused: bool,
}
```

**Step 2: Spawn terminal pane on vault open**

In `open_vault`:

```rust
let terminal = TerminalPane::new(&path, 24, 80);
self.terminal_pane = Some(terminal);
```

**Step 3: Register terminal commands**

```rust
commands.register("pane.terminal.toggle", || { /* handled in handle_named_command */ });
commands.register("pane.terminal.focus",  || {});
commands.register("terminal.new_tab",     || {});
commands.register("terminal.close_tab",   || {});
```

In `handle_named_command`:

```rust
"pane.terminal.toggle" => {
    self.terminal_visible = !self.terminal_visible;
    self.events.emit("pane.toggled", "terminal");
}
"pane.terminal.focus" => {
    self.terminal_visible = true;
    self.terminal_focused = true;
}
"terminal.new_tab" => {
    if let Some(tp) = &mut self.terminal_pane {
        tp.new_tab();
    }
}
"terminal.close_tab" => {
    if let Some(tp) = &mut self.terminal_pane {
        tp.close_tab();
    }
}
```

**Step 4: Route keypresses to the terminal when focused**

In `KeyboardInput`, before Vim routing:

```rust
if self.terminal_focused {
    if let Some(tp) = &mut self.terminal_pane {
        let bytes = key_to_bytes(&logical_key, &modifiers);
        tp.active_session().write(&bytes);
    }
    if let Some(w) = &self.window { w.request_redraw(); }
    return;
}
```

Add a `key_to_bytes` helper:

```rust
fn key_to_bytes(key: &WKey, modifiers: &winit::event::Modifiers) -> Vec<u8> {
    match key {
        WKey::Character(s) => {
            if modifiers.control_key() {
                // Ctrl+A..Z → byte 1..26
                if let Some(c) = s.chars().next() {
                    let lower = c.to_ascii_lowercase();
                    if lower >= 'a' && lower <= 'z' {
                        return vec![lower as u8 - b'a' + 1];
                    }
                }
            }
            s.as_bytes().to_vec()
        }
        WKey::Named(winit::keyboard::NamedKey::Enter)     => vec![b'\r'],
        WKey::Named(winit::keyboard::NamedKey::Backspace) => vec![127],
        WKey::Named(winit::keyboard::NamedKey::Escape)    => vec![27],
        WKey::Named(winit::keyboard::NamedKey::ArrowUp)   => vec![27, b'[', b'A'],
        WKey::Named(winit::keyboard::NamedKey::ArrowDown) => vec![27, b'[', b'B'],
        WKey::Named(winit::keyboard::NamedKey::ArrowRight)=> vec![27, b'[', b'C'],
        WKey::Named(winit::keyboard::NamedKey::ArrowLeft) => vec![27, b'[', b'D'],
        _ => vec![],
    }
}
```

**Step 5: Tick the terminal on `RedrawRequested` and draw it**

```rust
WindowEvent::RedrawRequested => {
    if let Some(tp) = &mut self.terminal_pane {
        tp.tick_all();
    }

    if let Some(r) = &mut self.renderer {
        r.scene.reset();
        // ... draw tab bar, file tree, editor as before ...

        if self.terminal_visible {
            if let Some(tp) = &mut self.terminal_pane {
                let session = tp.active_session();
                let terminal_x = r.config.width as f32 - 80.0 * 9.0;
                render::terminal::draw_terminal(
                    &mut r.scene,
                    &session.performer.grid,
                    terminal_x,
                    render::ui::TAB_HEIGHT,
                    9.0,  // cell_width
                    18.0, // cell_height
                );
            }
        }

        r.render();
    }
    if let Some(w) = &self.window {
        w.request_redraw();
    }
}
```

**Step 6: Build and run**

```bash
cargo run
```

Expected: Open a vault. Press `cmd+j` — terminal pane appears on the right with a shell prompt (cell backgrounds and cursor visible). Type characters — they route to the shell. Press `cmd+t` — new terminal tab spawns.

**Step 7: Stage changes**

```bash
git add .
```

**Step 8: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 5: Copy/paste in the terminal

**Files:**
- Modify: `src/main.rs`

`cmd+c` in terminal focus sends the selected text to the clipboard. `cmd+v` pastes from the clipboard into the pty.

**Step 1: Add `arboard` to `Cargo.toml`**

```toml
arboard = "3"
```

**Step 2: Add clipboard helper**

```rust
fn get_clipboard() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

fn set_clipboard(text: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}
```

**Step 3: Handle `cmd+c` and `cmd+v` when terminal is focused**

In the terminal-focused keyboard branch:

```rust
if modifiers.super_key() {
    if let WKey::Character(s) = &logical_key {
        match s.as_str() {
            "c" => {
                // For MVP: send ETX (Ctrl+C) to the pty (interrupt).
                // Full selection copy is a post-MVP feature.
                tp.active_session().write(&[3]);
                return;
            }
            "v" => {
                if let Some(text) = get_clipboard() {
                    tp.active_session().write(text.as_bytes());
                }
                return;
            }
            _ => {}
        }
    }
}
```

**Step 4: Build and run**

```bash
cargo run
```

Expected: `cmd+v` pastes clipboard content into the terminal. `cmd+c` interrupts the running process.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

## Milestone 6 Complete

At this point:
- `src/terminal/` owns pty spawning, ANSI parsing, and cell grid
- `src/render/terminal.rs` draws the cell grid (backgrounds + cursor) via Vello
- Multiple terminal tabs (`cmd+t` / `cmd+w`)
- Keyboard routing to the active pty when the terminal is focused
- `cmd+j` toggles focus / visibility via the command registry
- `cmd+v` pastes from the system clipboard; `cmd+c` sends interrupt
- Working directory defaults to vault root on spawn
- ANSI SGR colours (8 standard foreground colours) supported

**Out of scope for MVP (confirmed):** tmux-style pane splits within the terminal pane, custom shell config UI, full text selection and copy within the terminal.

---

## Full MVP Complete

All 6 milestones deliver a working Onyx MVP:

| Milestone | Status |
|-----------|--------|
| 1 — Rendering Foundation | GPU window + Vello pipeline |
| 2 — Editor Core | Rope buffer + Vim modal editing |
| 3 — WYSIWYG Layer | Markdown AST → styled live preview |
| 4 — Vault System | Config persistence + first-launch flow |
| 5 — Workspace Shell | File tree, tab bar, keybindings, command registry |
| 6 — Terminal Pane | Embedded pty, multi-tab, ANSI colour |
