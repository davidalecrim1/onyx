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
    /// Parses the full text into a typed AST. Returns an empty document on empty input.
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
                    if in_list {
                        list_items.push(inlines);
                    } else {
                        blocks.push(Block::Paragraph { inlines });
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
                    link_url = dest_url.to_string();
                    inline_stack.push(Vec::new());
                }
                Event::End(TagEnd::Link) => {
                    let inlines = inline_stack.pop().unwrap_or_default();
                    let text = inlines
                        .iter()
                        .map(|inline| match inline {
                            Inline::Text(t) => t.as_str(),
                            _ => "",
                        })
                        .collect::<String>();
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

    /// Returns the parsed block sequence.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
}

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
        let Block::Paragraph { inlines } = &doc.blocks()[0] else {
            panic!("not a paragraph")
        };
        assert!(inlines.iter().any(|inline| matches!(inline, Inline::Bold(_))));
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
