use crate::buffer::Buffer;
use crate::markdown::{Block, Document, Inline};
use crate::vim::{BufferCommand, Key, Mode, VimEngine};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    LivePreview,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStyle {
    Normal,
    /// Heading level 1–6.
    Heading(u8),
    Bold,
    Italic,
    Code,
    Link,
    BulletMarker,
    CodeBlockText,
}

#[derive(Debug, Clone)]
pub struct RenderSpan {
    pub text: String,
    pub style: SpanStyle,
    /// When true, render as plain syntax rather than styled output (cursor is inside construct).
    pub is_raw: bool,
}

#[derive(Debug, Clone)]
pub struct RenderLine {
    pub spans: Vec<RenderSpan>,
}

pub struct Tab {
    pub editor: Editor,
    pub view_mode: ViewMode,
    pub file_path: Option<std::path::PathBuf>,
    pub document: Document,
    dirty: bool,
}

impl Tab {
    /// Creates a tab pre-loaded with the given text, starting in Live Preview mode.
    pub fn new(text: &str) -> Self {
        Tab {
            document: Document::parse(text),
            editor: Editor::new(text),
            view_mode: ViewMode::LivePreview,
            file_path: None,
            dirty: false,
        }
    }

    /// Marks the document as needing a re-parse on the next sync.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Re-parses the document from the buffer only when the buffer has changed since last sync.
    pub fn sync_document(&mut self) {
        if self.dirty {
            self.document = Document::parse(&self.editor.buffer_text());
            self.dirty = false;
        }
    }

    /// Toggles between Live Preview and Raw mode.
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::LivePreview => ViewMode::Raw,
            ViewMode::Raw => ViewMode::LivePreview,
        };
    }
}

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

    /// Converts a Document AST into a flat list of styled lines the renderer consumes.
    pub fn build_render_lines(&self, doc: &Document, mode: ViewMode) -> Vec<RenderLine> {
        if mode == ViewMode::Raw {
            return (0..self.buffer.line_count())
                .map(|idx| RenderLine {
                    spans: vec![RenderSpan {
                        text: self.buffer.line(idx),
                        style: SpanStyle::Normal,
                        is_raw: true,
                    }],
                })
                .collect();
        }

        let mut lines: Vec<RenderLine> = Vec::new();

        for block in doc.blocks() {
            match block {
                Block::Heading { level, inlines } => {
                    let text = inlines.iter().map(inline_text).collect::<String>();
                    lines.push(RenderLine {
                        spans: vec![RenderSpan { text, style: SpanStyle::Heading(*level), is_raw: false }],
                    });
                }
                Block::Paragraph { inlines } => {
                    let spans = inlines.iter().map(|inline| {
                        let (text, style) = inline_style(inline);
                        RenderSpan { text, style, is_raw: false }
                    }).collect();
                    lines.push(RenderLine { spans });
                }
                Block::CodeBlock { code, .. } => {
                    for code_line in code.lines() {
                        lines.push(RenderLine {
                            spans: vec![RenderSpan {
                                text: code_line.to_string(),
                                style: SpanStyle::CodeBlockText,
                                is_raw: false,
                            }],
                        });
                    }
                }
                Block::List(items) => {
                    for item_inlines in items {
                        let mut spans = vec![RenderSpan {
                            text: "• ".to_string(),
                            style: SpanStyle::BulletMarker,
                            is_raw: false,
                        }];
                        for inline in item_inlines {
                            let (text, style) = inline_style(inline);
                            spans.push(RenderSpan { text, style, is_raw: false });
                        }
                        lines.push(RenderLine { spans });
                    }
                }
                Block::ThematicBreak => {
                    lines.push(RenderLine {
                        spans: vec![RenderSpan {
                            text: "───────────────────".to_string(),
                            style: SpanStyle::Normal,
                            is_raw: false,
                        }],
                    });
                }
            }
        }

        lines
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

fn inline_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(t) | Inline::Bold(t) | Inline::Italic(t) | Inline::Code(t) => t.clone(),
        Inline::Link { text, .. } => text.clone(),
    }
}

fn inline_style(inline: &Inline) -> (String, SpanStyle) {
    match inline {
        Inline::Text(t) => (t.clone(), SpanStyle::Normal),
        Inline::Bold(t) => (t.clone(), SpanStyle::Bold),
        Inline::Italic(t) => (t.clone(), SpanStyle::Italic),
        Inline::Code(t) => (t.clone(), SpanStyle::Code),
        Inline::Link { text, .. } => (text.clone(), SpanStyle::Link),
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
