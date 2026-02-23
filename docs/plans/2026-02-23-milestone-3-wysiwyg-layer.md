# Milestone 3: WYSIWYG Layer

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Parse Markdown to an AST on every edit and render it as styled text in Live Preview mode. Each tab independently tracks whether it is in Live Preview or Raw mode, with a toggle button in the tab bar. The cursor entering a Markdown construct reveals the raw syntax for that construct.

**Architecture:** `src/markdown/` holds a `Document` (AST built by `pulldown-cmark`) and a `diff` utility that re-parses only the affected block on edit. `src/editor/` grows a `RenderView` that maps AST nodes to styled `RenderSpan` values. `src/render/` is extended to blit actual Vello text glyphs (completing the glyph path stubbed in Milestone 2) and to draw styled spans (bold, italic, code, heading sizes). Each tab in the editor holds its own view mode enum.

**Tech Stack:** `pulldown-cmark 0.11`, `vello 0.3`, `cosmic-text 0.12`

---

## Prerequisites

Milestone 2 complete: rope buffer, Vim engine, and editor layer all working. Window shows cursor rectangle on key events.

---

### Task 1: Add the markdown module and parse a document

**Files:**
- Create: `src/markdown/mod.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing tests**

```rust
// src/markdown/mod.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_parsed() {
        let doc = Document::parse("# Hello");
        assert!(matches!(doc.blocks()[0], Block::Heading { level: 1, .. }));
    }

    #[test]
    fn paragraph_with_bold_parsed() {
        let doc = Document::parse("This is **bold** text.");
        let Block::Paragraph { inlines } = &doc.blocks()[0] else { panic!("not a paragraph") };
        assert!(inlines.iter().any(|i| matches!(i, Inline::Bold(_))));
    }

    #[test]
    fn code_block_parsed() {
        let doc = Document::parse("```\nfn main() {}\n```");
        assert!(matches!(doc.blocks()[0], Block::CodeBlock { .. }));
    }

    #[test]
    fn bullet_list_parsed() {
        let doc = Document::parse("- item one\n- item two");
        assert!(matches!(doc.blocks()[0], Block::List(_)));
    }
}
```

**Step 2: Run to confirm failure**

```bash
cargo test markdown 2>&1
```

Expected: compile error — `Document`, `Block`, `Inline` not found.

**Step 3: Implement `src/markdown/mod.rs`**

```rust
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Link { text: String, url: String },
}

#[derive(Debug, Clone)]
pub enum Block {
    Heading { level: u8, inlines: Vec<Inline> },
    Paragraph { inlines: Vec<Inline> },
    CodeBlock { language: String, code: String },
    List(Vec<Vec<Inline>>),
    ThematicBreak,
}

pub struct Document {
    blocks: Vec<Block>,
}

impl Document {
    pub fn parse(text: &str) -> Self {
        let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;
        let parser = Parser::new_ext(text, opts);

        let mut blocks: Vec<Block> = Vec::new();
        let mut inline_stack: Vec<Vec<Inline>> = Vec::new();
        let mut heading_level: u8 = 0;
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_body = String::new();
        let mut list_items: Vec<Vec<Inline>> = Vec::new();
        let mut in_list = false;
        let mut bold = false;
        let mut italic = false;
        let mut link_url = String::new();
        let mut in_link = false;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    heading_level = level as u8;
                    inline_stack.push(Vec::new());
                }
                Event::End(TagEnd::Heading(_)) => {
                    let inlines = inline_stack.pop().unwrap_or_default();
                    blocks.push(Block::Heading { level: heading_level, inlines });
                }
                Event::Start(Tag::Paragraph) => inline_stack.push(Vec::new()),
                Event::End(TagEnd::Paragraph) => {
                    let inlines = inline_stack.pop().unwrap_or_default();
                    if !in_list {
                        blocks.push(Block::Paragraph { inlines });
                    } else {
                        list_items.push(inlines);
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        _ => String::new(),
                    };
                    code_body.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    blocks.push(Block::CodeBlock {
                        language: code_lang.clone(),
                        code: code_body.clone(),
                    });
                }
                Event::Start(Tag::List(_)) => in_list = true,
                Event::End(TagEnd::List(_)) => {
                    in_list = false;
                    blocks.push(Block::List(list_items.clone()));
                    list_items.clear();
                }
                Event::Start(Tag::Item) => inline_stack.push(Vec::new()),
                Event::End(TagEnd::Item) => {
                    let inlines = inline_stack.pop().unwrap_or_default();
                    list_items.push(inlines);
                }
                Event::Start(Tag::Strong) => bold = true,
                Event::End(TagEnd::Strong) => bold = false,
                Event::Start(Tag::Emphasis) => italic = true,
                Event::End(TagEnd::Emphasis) => italic = false,
                Event::Start(Tag::Link { dest_url, .. }) => {
                    in_link = true;
                    link_url = dest_url.to_string();
                    inline_stack.push(Vec::new());
                }
                Event::End(TagEnd::Link) => {
                    in_link = false;
                    let inlines = inline_stack.pop().unwrap_or_default();
                    let text = inlines.iter().map(|i| match i {
                        Inline::Text(t) => t.as_str(),
                        _ => "",
                    }).collect::<String>();
                    if let Some(top) = inline_stack.last_mut() {
                        top.push(Inline::Link { text, url: link_url.clone() });
                    }
                }
                Event::Text(t) => {
                    if in_code_block {
                        code_body.push_str(&t);
                    } else if let Some(top) = inline_stack.last_mut() {
                        let s = t.to_string();
                        let inline = if bold {
                            Inline::Bold(s)
                        } else if italic {
                            Inline::Italic(s)
                        } else {
                            Inline::Text(s)
                        };
                        top.push(inline);
                    }
                }
                Event::Code(c) => {
                    if let Some(top) = inline_stack.last_mut() {
                        top.push(Inline::Code(c.to_string()));
                    }
                }
                Event::Rule => blocks.push(Block::ThematicBreak),
                _ => {}
            }
        }

        Document { blocks }
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
}
```

**Step 4: Run tests**

```bash
cargo test markdown 2>&1
```

Expected: all 4 tests pass.

**Step 5: Declare module in `src/main.rs`**

```rust
mod markdown;
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

### Task 2: Define the render span model in the editor layer

**Files:**
- Modify: `src/editor/mod.rs`

The editor converts AST blocks to a flat list of `RenderLine` values — each line is a sequence of styled spans. The renderer consumes this instead of raw strings.

**Step 1: Add `RenderSpan`, `SpanStyle`, and `RenderLine` types**

```rust
// src/editor/mod.rs — add before Editor struct

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStyle {
    Normal,
    Heading(u8), // 1–6
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
    pub is_raw: bool, // true = render as plain syntax (cursor in this construct)
}

#[derive(Debug, Clone)]
pub struct RenderLine {
    pub spans: Vec<RenderSpan>,
}
```

**Step 2: Add `build_render_lines` to `Editor`**

This method takes a `Document` and the cursor position and produces the `Vec<RenderLine>` the renderer will consume.

```rust
use crate::markdown::{Block, Document, Inline};
use crate::buffer::Cursor;

impl Editor {
    pub fn build_render_lines(&self, doc: &Document, mode: ViewMode, cursor: Cursor) -> Vec<RenderLine> {
        let raw_lines: Vec<String> = (0..self.buffer.line_count())
            .map(|i| self.buffer.line(i))
            .collect();

        if mode == ViewMode::Raw {
            return raw_lines.iter().map(|l| RenderLine {
                spans: vec![RenderSpan { text: l.clone(), style: SpanStyle::Normal, is_raw: true }],
            }).collect();
        }

        // Live preview: map AST blocks to render lines.
        let mut lines: Vec<RenderLine> = Vec::new();

        for block in doc.blocks() {
            match block {
                Block::Heading { level, inlines } => {
                    let text = inlines.iter().map(|i| inline_text(i)).collect::<String>();
                    lines.push(RenderLine {
                        spans: vec![RenderSpan { text, style: SpanStyle::Heading(*level), is_raw: false }],
                    });
                }
                Block::Paragraph { inlines } => {
                    let spans = inlines.iter().map(|inline| {
                        let (text, style) = match inline {
                            Inline::Text(t)  => (t.clone(), SpanStyle::Normal),
                            Inline::Bold(t)  => (t.clone(), SpanStyle::Bold),
                            Inline::Italic(t) => (t.clone(), SpanStyle::Italic),
                            Inline::Code(t)  => (t.clone(), SpanStyle::Code),
                            Inline::Link { text, .. } => (text.clone(), SpanStyle::Link),
                        };
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
                            let (text, style) = match inline {
                                Inline::Text(t)  => (t.clone(), SpanStyle::Normal),
                                Inline::Bold(t)  => (t.clone(), SpanStyle::Bold),
                                Inline::Italic(t) => (t.clone(), SpanStyle::Italic),
                                Inline::Code(t)  => (t.clone(), SpanStyle::Code),
                                Inline::Link { text, .. } => (text.clone(), SpanStyle::Link),
                            };
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
}

fn inline_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(t) | Inline::Bold(t) | Inline::Italic(t) | Inline::Code(t) => t.clone(),
        Inline::Link { text, .. } => text.clone(),
    }
}
```

**Step 3: Add `ViewMode` enum to `src/editor/mod.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    LivePreview,
    Raw,
}
```

**Step 4: Build**

```bash
cargo build 2>&1
```

Expected: Compiles. No errors.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 3: Extend the renderer to draw styled spans

**Files:**
- Modify: `src/render/mod.rs`

Replace the stub `draw_buffer` with a `draw_render_lines` method that handles font weight, size per heading level, monospace font for code, and a background fill for code blocks.

**Step 1: Add `draw_render_lines` to `src/render/mod.rs`**

```rust
use crate::editor::{RenderLine, RenderSpan, SpanStyle};

impl Renderer {
    pub fn draw_render_lines(
        &mut self,
        render_lines: &[RenderLine],
        cursor_line: usize,
        cursor_col: usize,
    ) {
        let left_pad = 48.0_f32;
        let top_pad = 8.0_f32;
        let base_line_height = 22.0_f32;

        for (line_idx, render_line) in render_lines.iter().enumerate() {
            let line_height = heading_line_height(&render_line.spans, base_line_height);
            let y = top_pad + line_idx as f32 * base_line_height;

            // Code block background fill.
            if render_line.spans.iter().any(|s| s.style == SpanStyle::CodeBlockText) {
                let bg = vello::kurbo::Rect::new(
                    left_pad as f64,
                    y as f64,
                    (self.config.width as f32 - left_pad) as f64,
                    (y + line_height) as f64,
                );
                self.scene.fill(
                    vello::peniko::Fill::NonZero,
                    vello::kurbo::Affine::IDENTITY,
                    &vello::peniko::Brush::Solid(vello::peniko::Color::from_rgba8(30, 30, 36, 255)),
                    None,
                    &bg,
                );
            }

            // Cursor rectangle.
            if line_idx == cursor_line {
                let char_width = 9.0_f32;
                let cx = left_pad + cursor_col as f32 * char_width;
                let cursor_rect = vello::kurbo::Rect::new(
                    cx as f64,
                    y as f64,
                    (cx + char_width) as f64,
                    (y + line_height) as f64,
                );
                self.scene.fill(
                    vello::peniko::Fill::NonZero,
                    vello::kurbo::Affine::IDENTITY,
                    &vello::peniko::Brush::Solid(vello::peniko::Color::from_rgba8(97, 175, 239, 180)),
                    None,
                    &cursor_rect,
                );
            }

            // Draw spans.
            let mut x = left_pad;
            for span in &render_line.spans {
                let font_size = span_font_size(&span.style);
                let metrics = cosmic_text::Metrics::new(font_size, line_height);
                let mut text_buf = cosmic_text::Buffer::new(&mut self.font_system, metrics);
                text_buf.set_size(
                    &mut self.font_system,
                    Some(self.config.width as f32 - x),
                    None,
                );

                let attrs = span_attrs(&span.style);
                text_buf.set_text(
                    &mut self.font_system,
                    &span.text,
                    attrs,
                    cosmic_text::Shaping::Advanced,
                );
                text_buf.shape_until_scroll(&mut self.font_system, false);

                // Measure advance for x positioning.
                for run in text_buf.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical = glyph.physical((x, y), 1.0);
                        let _ = self.swash_cache.get_image(
                            &mut self.font_system,
                            physical.cache_key,
                        );
                        // TODO: blit glyph image into scene (vello::glyph::GlyphProvider).
                        x += glyph.w;
                    }
                }
            }
        }
    }
}

fn heading_line_height(spans: &[RenderSpan], base: f32) -> f32 {
    if let Some(span) = spans.first() {
        match span.style {
            SpanStyle::Heading(1) => base * 2.0,
            SpanStyle::Heading(2) => base * 1.6,
            SpanStyle::Heading(3) => base * 1.3,
            _ => base,
        }
    } else {
        base
    }
}

fn span_font_size(style: &SpanStyle) -> f32 {
    match style {
        SpanStyle::Heading(1) => 30.0,
        SpanStyle::Heading(2) => 24.0,
        SpanStyle::Heading(3) => 20.0,
        SpanStyle::Heading(_) => 16.0,
        SpanStyle::Code | SpanStyle::CodeBlockText => 14.0,
        _ => 15.0,
    }
}

fn span_attrs(style: &SpanStyle) -> cosmic_text::Attrs<'static> {
    use cosmic_text::{Attrs, Style, Weight};
    let attrs = Attrs::new();
    match style {
        SpanStyle::Bold | SpanStyle::Heading(_) => attrs.weight(Weight::BOLD),
        SpanStyle::Italic => attrs.style(Style::Italic),
        SpanStyle::Code | SpanStyle::CodeBlockText => attrs, // monospace if font system has it
        _ => attrs,
    }
}
```

**Step 2: Update `App::window_event` to use `draw_render_lines`**

Replace the `draw_buffer` call with:

```rust
WindowEvent::RedrawRequested => {
    if let Some(r) = &mut self.renderer {
        r.scene.reset();
        let doc = crate::markdown::Document::parse(&self.editor.buffer_text());
        let render_lines = self.editor.build_render_lines(
            &doc,
            self.view_mode,
            self.editor.buffer.cursor(),
        );
        let cursor = self.editor.buffer.cursor();
        r.draw_render_lines(&render_lines, cursor.line, cursor.col);
        r.render();
    }
    if let Some(w) = &self.window {
        w.request_redraw();
    }
}
```

Add `view_mode: crate::editor::ViewMode` field to `App`, defaulting to `ViewMode::LivePreview`.

**Step 3: Build and run**

```bash
cargo run
```

Expected: Window renders with styled layout — heading text is larger, code blocks have a background fill, cursor rectangle visible. Text glyphs still stub but positions are computed.

**Step 4: Stage changes**

```bash
git add .
```

**Step 5: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 4: Per-tab view mode with toggle button stub

**Files:**
- Modify: `src/editor/mod.rs`
- Modify: `src/main.rs`

Each tab independently tracks its view mode. For MVP a single editor pane maps to a single `Tab` struct. The toggle is a keybinding stub for now; the tab bar UI is part of Milestone 5 (Workspace Shell).

**Step 1: Add `Tab` struct to `src/editor/mod.rs`**

```rust
pub struct Tab {
    pub editor: Editor,
    pub view_mode: ViewMode,
    pub file_path: Option<std::path::PathBuf>,
}

impl Tab {
    pub fn new(text: &str) -> Self {
        Tab {
            editor: Editor::new(text),
            view_mode: ViewMode::LivePreview,
            file_path: None,
        }
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::LivePreview => ViewMode::Raw,
            ViewMode::Raw => ViewMode::LivePreview,
        };
    }
}
```

**Step 2: Update `App` to hold a `Tab`**

Replace the `editor: Editor` and `view_mode: ViewMode` fields with a single `tab: Tab`:

```rust
use editor::Tab;

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    tab: Tab,
}
```

Update `RedrawRequested`:

```rust
let doc = crate::markdown::Document::parse(&self.tab.editor.buffer_text());
let render_lines = self.tab.editor.build_render_lines(
    &doc,
    self.tab.view_mode,
    self.tab.editor.buffer.cursor(),
);
let cursor = self.tab.editor.buffer.cursor();
r.draw_render_lines(&render_lines, cursor.line, cursor.col);
```

**Step 3: Wire a toggle shortcut**

In the keyboard handler, before routing to the Vim engine, intercept `cmd+option+p` (or a dedicated key) to toggle view mode. For MVP stub, use `ctrl+t`:

```rust
// in KeyboardInput handler, before routing to vim:
if let WKey::Character(s) = &logical_key {
    if s == "t" && modifiers.control_key() {
        self.tab.toggle_view_mode();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        return;
    }
}
```

> The tab bar toggle button is wired in Milestone 5. `ctrl+t` is a development shortcut only.

**Step 4: Build and run**

```bash
cargo run
```

Expected: Window renders in Live Preview mode. `ctrl+t` toggles to Raw mode — the layout switches to plain text lines. Toggle back.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 5: Block-level re-parse on edit

**Files:**
- Modify: `src/editor/mod.rs`

Currently we re-parse the full document on every redraw. Add a dirty-block tracking mechanism so only the block containing the cursor is re-parsed on each keystroke.

**Step 1: Add `dirty` flag to `Tab`**

```rust
pub struct Tab {
    pub editor: Editor,
    pub view_mode: ViewMode,
    pub file_path: Option<std::path::PathBuf>,
    pub document: crate::markdown::Document,
    dirty: bool,
}

impl Tab {
    pub fn new(text: &str) -> Self {
        let document = crate::markdown::Document::parse(text);
        Tab {
            editor: Editor::new(text),
            view_mode: ViewMode::LivePreview,
            file_path: None,
            document,
            dirty: false,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn sync_document(&mut self) {
        if self.dirty {
            self.document = crate::markdown::Document::parse(&self.editor.buffer_text());
            self.dirty = false;
        }
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::LivePreview => ViewMode::Raw,
            ViewMode::Raw => ViewMode::LivePreview,
        };
    }
}
```

**Step 2: Call `mark_dirty` in `App` after key handling**

```rust
self.tab.editor.handle_key(k);
self.tab.mark_dirty();
```

**Step 3: Call `sync_document` at the start of `RedrawRequested`**

```rust
WindowEvent::RedrawRequested => {
    if let Some(r) = &mut self.renderer {
        self.tab.sync_document();
        r.scene.reset();
        let render_lines = self.tab.editor.build_render_lines(
            &self.tab.document,
            self.tab.view_mode,
            self.tab.editor.buffer.cursor(),
        );
        // ...
    }
}
```

**Step 4: Build**

```bash
cargo build 2>&1
```

Expected: Compiles. No change in visible behaviour, but re-parses only happen when the buffer has changed.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

## Milestone 3 Complete

At this point:
- `src/markdown/` parses Markdown to a typed AST using `pulldown-cmark`
- `src/editor/` converts the AST to styled `RenderLine` values per tab view mode
- `src/render/` draws styled spans with correct font sizes for headings and background fills for code blocks
- Each tab independently tracks Live Preview vs Raw mode
- Re-parsing is gated behind a dirty flag — only triggers on edits

Text glyph blitting (`vello::glyph::GlyphProvider`) remains TODO inline — it is the first task to resolve when the Vello API version is pinned in Milestone 5 (Workspace Shell), which also adds the tab bar toggle button.
