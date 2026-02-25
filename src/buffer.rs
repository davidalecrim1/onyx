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
    scroll_offset: usize,
}

impl Buffer {
    /// Initialises the buffer from a string slice; cursor starts at the top-left.
    pub fn new(text: &str) -> Self {
        Buffer {
            rope: Rope::from_str(text),
            cursor: Cursor { line: 0, col: 0 },
            selection: None,
            scroll_offset: 0,
        }
    }

    /// Returns the index of the topmost visible line.
    pub fn scroll_offset(&self) -> usize { self.scroll_offset }

    /// Clamps scroll_offset so the cursor stays within a 3-line margin from both edges.
    /// `viewport_lines` is the number of fully visible lines in the editor pane.
    pub fn clamp_scroll(&mut self, viewport_lines: usize) {
        let margin = 3;
        let line = self.cursor.line;
        // Scroll up: cursor above top margin
        if line < self.scroll_offset + margin {
            self.scroll_offset = line.saturating_sub(margin);
        }
        // Scroll down: cursor too close to bottom
        if line + margin >= self.scroll_offset + viewport_lines {
            self.scroll_offset = line + margin + 1 - viewport_lines;
        }
        let max_offset = self.rope.len_lines().saturating_sub(1);
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }

    /// Returns a copy of the current cursor position.
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Returns the active visual selection, if any.
    pub fn selection(&self) -> Option<Selection> {
        self.selection
    }

    /// Ropey counts a trailing newline as an extra empty line; callers should account for this.
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Returns the line at `idx` including any trailing newline character.
    pub fn line(&self, idx: usize) -> String {
        self.rope.line(idx).to_string()
    }

    /// Inserts text at the cursor and advances the cursor past the inserted characters.
    pub fn insert(&mut self, text: &str) {
        let char_idx = self.rope.line_to_char(self.cursor.line) + self.cursor.col;
        self.rope.insert(char_idx, text);
        for ch in text.chars() {
            if ch == '\n' {
                self.cursor.line += 1;
                self.cursor.col = 0;
            } else {
                self.cursor.col += 1;
            }
        }
    }

    /// Deletes the character immediately before the cursor, joining lines if the cursor is at col 0.
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

    /// Deletes the character under the cursor, clamping col to the new line length afterward.
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

    /// Moves left without crossing line boundaries.
    pub fn move_left(&mut self) {
        self.cursor.col = self.cursor.col.saturating_sub(1);
    }

    /// Moves right without crossing into the newline character at the end of the line.
    pub fn move_right(&mut self) {
        let line_len = self.rope.line(self.cursor.line).len_chars();
        let max = if line_len > 0 { line_len - 1 } else { 0 };
        if self.cursor.col < max {
            self.cursor.col += 1;
        }
    }

    /// Moves up one line, clamping col to the new line's last valid position.
    pub fn move_up(&mut self) {
        if self.cursor.line > 0 {
            self.cursor.line -= 1;
            let line_len = self.rope.line(self.cursor.line).len_chars();
            let max = if line_len > 0 { line_len - 1 } else { 0 };
            self.cursor.col = self.cursor.col.min(max);
        }
    }

    /// Moves down one line, clamping col to the new line's last valid position.
    pub fn move_down(&mut self) {
        if self.cursor.line + 1 < self.rope.len_lines() {
            self.cursor.line += 1;
            let line_len = self.rope.line(self.cursor.line).len_chars();
            let max = if line_len > 0 { line_len - 1 } else { 0 };
            self.cursor.col = self.cursor.col.min(max);
        }
    }

    /// Moves to the first character of the current line.
    pub fn move_line_start(&mut self) {
        self.cursor.col = 0;
    }

    /// Moves to the last character of the current line, excluding the newline.
    pub fn move_line_end(&mut self) {
        let line_len = self.rope.line(self.cursor.line).len_chars();
        self.cursor.col = if line_len > 0 { line_len - 1 } else { 0 };
    }

    /// Advances to the start of the next word, staying on the current line.
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

    /// Moves back to the start of the previous word, staying on the current line.
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

    /// Jumps to line 0, col 0 (Vim `gg`).
    pub fn move_first_line(&mut self) {
        self.cursor.line = 0;
        self.cursor.col = 0;
    }

    /// Jumps to the last line of the buffer (Vim `G`).
    pub fn move_last_line(&mut self) {
        self.cursor.line = self.rope.len_lines().saturating_sub(1);
        self.cursor.col = 0;
    }

    /// Jumps to the next blank line, or the last line if none exists.
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

    /// Jumps to the previous blank line, or line 0 if none exists.
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

    /// Begins a visual selection with both anchor and head at the current cursor.
    pub fn start_visual(&mut self) {
        self.selection = Some(Selection { anchor: self.cursor, head: self.cursor });
    }

    /// Updates the moving end of the selection to track the current cursor.
    pub fn update_visual_head(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.head = self.cursor;
        }
    }

    /// Clears the active selection without modifying the buffer.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Returns the selected text as a string; handles reversed selections (head before anchor).
    pub fn yank_selection(&self) -> String {
        match self.selection {
            None => String::new(),
            Some(sel) => {
                let start_char = self.rope.line_to_char(sel.anchor.line) + sel.anchor.col;
                let end_char = self.rope.line_to_char(sel.head.line) + sel.head.col + 1;
                let (s, e) = if start_char <= end_char {
                    (start_char, end_char)
                } else {
                    (end_char, start_char)
                };
                self.rope.slice(s..e).to_string()
            }
        }
    }

    /// Yanks and removes the selected range, placing the cursor at the earlier endpoint.
    pub fn delete_selection(&mut self) -> String {
        let yanked = self.yank_selection();
        if let Some(sel) = self.selection {
            let start_char = self.rope.line_to_char(sel.anchor.line) + sel.anchor.col;
            let end_char = self.rope.line_to_char(sel.head.line) + sel.head.col + 1;
            let (s, e) = if start_char <= end_char {
                (start_char, end_char)
            } else {
                (end_char, start_char)
            };
            self.rope.remove(s..e);
            self.cursor = if start_char <= end_char { sel.anchor } else { sel.head };
        }
        self.selection = None;
        yanked
    }

    /// Inserts `text` at the cursor; alias for `insert` used at call sites that hold yanked text.
    pub fn paste(&mut self, text: &str) {
        self.insert(text);
    }
}

impl ToString for Buffer {
    fn to_string(&self) -> String {
        self.rope.to_string()
    }
}

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

    #[test]
    fn scroll_clamps_down() {
        let text: String = (0..20).map(|i| format!("line{}\n", i)).collect();
        let mut buf = Buffer::new(&text);
        for _ in 0..10 {
            buf.move_down();
        }
        buf.clamp_scroll(5);
        // cursor=10, margin=3, viewport=5 → scroll_offset = 10 + 3 + 1 - 5 = 9
        assert_eq!(buf.scroll_offset(), 9);
    }

    #[test]
    fn scroll_clamps_up() {
        let text: String = (0..20).map(|i| format!("line{}\n", i)).collect();
        let mut buf = Buffer::new(&text);
        for _ in 0..10 {
            buf.move_down();
        }
        buf.clamp_scroll(5);
        for _ in 0..9 {
            buf.move_up();
        }
        buf.clamp_scroll(5);
        assert_eq!(buf.scroll_offset(), 0);
    }
}
