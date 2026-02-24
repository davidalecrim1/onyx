use crate::buffer::Buffer;
use crate::vim::{BufferCommand, Key, Mode, VimEngine};

pub struct Editor {
    pub buffer: Buffer,
    pub vim: VimEngine,
}

impl Editor {
    /// Creates an editor pre-loaded with the given text, starting in Normal mode.
    pub fn new(text: &str) -> Self {
        Editor {
            buffer: Buffer::new(text),
            vim: VimEngine::new(),
        }
    }

    /// Returns the current Vim mode.
    pub fn mode(&self) -> Mode {
        self.vim.mode()
    }

    /// Returns the full buffer contents as a string.
    pub fn buffer_text(&self) -> String {
        self.buffer.to_string()
    }

    /// Forwards a key to the Vim engine and applies any resulting command to the buffer.
    pub fn handle_key(&mut self, key: Key) {
        if let Some(cmd) = self.vim.handle_key(key) {
            self.apply(cmd);
        }
    }

    fn apply(&mut self, cmd: BufferCommand) {
        match cmd {
            BufferCommand::MoveLeft             => self.buffer.move_left(),
            BufferCommand::MoveRight            => self.buffer.move_right(),
            BufferCommand::MoveUp               => self.buffer.move_up(),
            BufferCommand::MoveDown             => self.buffer.move_down(),
            BufferCommand::MoveWordForward      => self.buffer.move_word_forward(),
            BufferCommand::MoveWordBack         => self.buffer.move_word_back(),
            BufferCommand::MoveWordEnd          => self.buffer.move_word_forward(),
            BufferCommand::MoveLineStart        => self.buffer.move_line_start(),
            BufferCommand::MoveLineEnd          => self.buffer.move_line_end(),
            BufferCommand::MoveFirstLine        => self.buffer.move_first_line(),
            BufferCommand::MoveLastLine         => self.buffer.move_last_line(),
            BufferCommand::MoveParagraphForward => self.buffer.move_paragraph_forward(),
            BufferCommand::MoveParagraphBack    => self.buffer.move_paragraph_back(),
            BufferCommand::Insert(c)            => self.buffer.insert(&c.to_string()),
            BufferCommand::InsertNewline        => self.buffer.insert("\n"),
            BufferCommand::DeleteBefore         => self.buffer.delete_before(),
            BufferCommand::DeleteCharAtCursor   => self.buffer.delete_char_at_cursor(),
            BufferCommand::DeleteLine           => self.delete_line(),
            BufferCommand::Yank                 => {
                let text = self.buffer.yank_selection();
                self.buffer.clear_selection();
                self.vim.set_register(text);
            }
            BufferCommand::Delete               => {
                let text = self.buffer.delete_selection();
                self.vim.set_register(text);
            }
            BufferCommand::Change               => {
                let text = self.buffer.delete_selection();
                self.vim.set_register(text);
            }
            BufferCommand::Paste(text)          => self.buffer.paste(&text),
            BufferCommand::StartVisual          => self.buffer.start_visual(),
            BufferCommand::StartVisualLine      => {
                self.buffer.move_line_start();
                self.buffer.start_visual();
                self.buffer.move_line_end();
                self.buffer.update_visual_head();
            }
            BufferCommand::ClearSelection       => self.buffer.clear_selection(),
            BufferCommand::Undo | BufferCommand::Redo => {}
        }
    }

    /// Selects and removes the entire current line, then deletes the trailing newline.
    fn delete_line(&mut self) {
        self.buffer.move_line_start();
        self.buffer.start_visual();
        self.buffer.move_line_end();
        self.buffer.update_visual_head();
        let text = self.buffer.delete_selection();
        self.vim.set_register(text);
        self.buffer.delete_char_at_cursor();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vim::Key;

    #[test]
    fn typing_in_insert_mode_modifies_buffer() {
        let mut ed = Editor::new("hello");
        ed.handle_key(Key::Char('i'));
        ed.handle_key(Key::Char('!'));
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
