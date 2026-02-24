use std::io::{Read, Write};
use crossbeam_channel::{unbounded, Receiver};

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
    /// Creates a blank grid of the given dimensions with the cursor at (0, 0).
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

    /// Returns the cell at the given grid position.
    pub fn cell(&self, row: usize, col: usize) -> Cell {
        self.cells[row * self.cols + col]
    }

    /// Writes a string into the grid, advancing the cursor and handling newlines.
    pub fn write_str(&mut self, text: &str) {
        for ch in text.chars() {
            self.write_char(ch);
        }
    }

    fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.cursor_col = 0;
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

    /// Applies SGR (Select Graphic Rendition) escape parameters to the current pen state.
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
                let mut iter = params.iter();
                let row = iter.next().and_then(|p| p.first().copied()).unwrap_or(1).saturating_sub(1) as usize;
                let col = iter.next().and_then(|p| p.first().copied()).unwrap_or(1).saturating_sub(1) as usize;
                self.grid.cursor_row = row.min(self.grid.rows - 1);
                self.grid.cursor_col = col.min(self.grid.cols - 1);
            }
            'J' => {
                let rows = self.grid.rows;
                let cols = self.grid.cols;
                self.grid = TerminalGrid::new(rows, cols);
            }
            _ => {}
        }
    }
}

pub struct TerminalSession {
    pub name: String,
    pub performer: VtePerformer,
    parser: vte::Parser,
    writer: Box<dyn Write + Send>,
    reader_rx: Receiver<Vec<u8>>,
}

impl TerminalSession {
    /// Spawns a shell process in a new pty under the given vault root directory.
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

    /// Drains pending output from the pty reader and advances the ANSI parser.
    pub fn tick(&mut self) {
        while let Ok(bytes) = self.reader_rx.try_recv() {
            for &b in &bytes {
                self.parser.advance(&mut self.performer, b);
            }
        }
    }

    /// Sends raw bytes to the pty input.
    pub fn write(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
    }
}

pub struct TerminalPane {
    sessions: Vec<TerminalSession>,
    active: usize,
    vault_root: std::path::PathBuf,
    rows: u16,
    cols: u16,
}

impl TerminalPane {
    /// Creates a pane with one initial shell session rooted at `vault_root`.
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

    /// Opens a new terminal tab and makes it active.
    pub fn new_tab(&mut self) {
        let name = format!("zsh {}", self.sessions.len() + 1);
        let session = TerminalSession::spawn(&name, &self.vault_root, self.rows, self.cols);
        self.sessions.push(session);
        self.active = self.sessions.len() - 1;
    }

    /// Closes the active tab; ignored when only one tab remains.
    pub fn close_tab(&mut self) {
        if self.sessions.len() > 1 {
            self.sessions.remove(self.active);
            if self.active >= self.sessions.len() {
                self.active = self.sessions.len() - 1;
            }
        }
    }

    /// Returns a mutable reference to the currently focused session.
    pub fn active_session(&mut self) -> &mut TerminalSession {
        &mut self.sessions[self.active]
    }

    /// Ticks all sessions to drain any pending pty output.
    pub fn tick_all(&mut self) {
        for session in &mut self.sessions {
            session.tick();
        }
    }

    /// Returns the display name of each tab in order.
    pub fn tab_names(&self) -> Vec<&str> {
        self.sessions.iter().map(|s| s.name.as_str()).collect()
    }

    /// Returns the index of the currently active tab.
    pub fn active_index(&self) -> usize {
        self.active
    }
}

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
