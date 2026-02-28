use std::collections::HashSet;
use std::path::PathBuf;

use taffy::style_helpers::{length, TaffyMaxContent};
use taffy::{FlexDirection, Size, Style, TaffyTree};

use crate::file_tree::{flatten_tree_filtered, scan_file_tree, FileTreeEntry};
use crate::text::draw_text;
use crate::ui::{DrawContext, HitId, HitSink, Panel, Rect};
use crate::vault::Vault;

const SIDEBAR_WIDTH: f32 = 220.0;
const FILE_ENTRY_HIT_BASE: u32 = 1000;
const ROW_HEIGHT: f32 = 22.0;

/// Minimal editor view with a file-tree sidebar and content area.
pub struct EditorView {
    vault_name: String,
    file_tree: Vec<FileTreeEntry>,
    selected_path: Option<PathBuf>,
    content_lines: Vec<String>,
    collapsed_dirs: HashSet<PathBuf>,
}

impl EditorView {
    /// Scans the vault's file tree on construction.
    pub fn new(vault: &Vault) -> Self {
        let file_tree = scan_file_tree(&vault.root).unwrap_or_default();
        Self {
            vault_name: vault.config.name.clone(),
            file_tree,
            selected_path: None,
            content_lines: Vec::new(),
            collapsed_dirs: HashSet::new(),
        }
    }

    /// Returns true if the hit id belongs to a file tree entry.
    pub fn is_file_hit(id: HitId) -> bool {
        id.0 >= FILE_ENTRY_HIT_BASE
    }

    /// Handles a click on a file tree entry.
    pub fn handle_click(&mut self, hit_id: HitId) {
        let index = (hit_id.0 - FILE_ENTRY_HIT_BASE) as usize;
        let flat = flatten_tree_filtered(&self.file_tree, &self.collapsed_dirs);

        let Some(entry) = flat.get(index) else {
            return;
        };

        if entry.is_directory {
            let path = entry.path.clone();
            if !self.collapsed_dirs.remove(&path) {
                self.collapsed_dirs.insert(path);
            }
        } else {
            let path = entry.path.clone();
            self.selected_path = Some(path.clone());
            self.content_lines = match std::fs::read(&path) {
                Ok(bytes) => match String::from_utf8(bytes) {
                    Ok(text) => text.lines().map(String::from).collect(),
                    Err(_) => vec!["Binary file \u{2014} cannot display".to_string()],
                },
                Err(error) => vec![format!("Error reading file: {error}")],
            };
        }
    }

    /// Draws the sidebar file tree and content area using Taffy for layout.
    pub fn render(
        &self,
        ctx: &mut DrawContext,
        hits: &mut HitSink,
        bounds: Rect,
    ) -> Result<(), taffy::TaffyError> {
        let mut tree: TaffyTree<()> = TaffyTree::new();

        let sidebar_node = tree.new_leaf(Style {
            size: Size {
                width: length(SIDEBAR_WIDTH),
                height: length(bounds.height),
            },
            ..Default::default()
        })?;

        let separator_node = tree.new_leaf(Style {
            size: Size {
                width: length(1.0),
                height: length(bounds.height),
            },
            ..Default::default()
        })?;

        let content_node = tree.new_leaf(Style {
            flex_grow: 1.0,
            size: Size {
                width: taffy::Dimension::auto(),
                height: length(bounds.height),
            },
            ..Default::default()
        })?;

        let root = tree.new_with_children(
            Style {
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: length(bounds.width),
                    height: length(bounds.height),
                },
                ..Default::default()
            },
            &[sidebar_node, separator_node, content_node],
        )?;

        tree.compute_layout(root, Size::MAX_CONTENT)?;

        let root_layout = tree.layout(root)?;
        let root_rect = Rect::from_layout(root_layout, bounds.x, bounds.y);

        let sidebar_layout = tree.layout(sidebar_node)?;
        let sidebar_rect = Rect::from_layout(sidebar_layout, root_rect.x, root_rect.y);

        let separator_layout = tree.layout(separator_node)?;
        let separator_rect = Rect::from_layout(separator_layout, root_rect.x, root_rect.y);

        let content_layout = tree.layout(content_node)?;
        let content_rect = Rect::from_layout(content_layout, root_rect.x, root_rect.y);

        Panel::new(sidebar_rect, ctx.theme.surface).paint(ctx.scene);

        let header_y = 16.0;
        draw_text(
            ctx.scene,
            ctx.text,
            &self.vault_name,
            20.0,
            (sidebar_rect.x + 12.0, sidebar_rect.y + header_y),
            ctx.theme.text_primary,
        );

        let flat = flatten_tree_filtered(&self.file_tree, &self.collapsed_dirs);
        let mut entry_y = sidebar_rect.y + header_y + 36.0;
        for (index, entry) in flat.iter().enumerate() {
            if entry_y > sidebar_rect.y + sidebar_rect.height {
                break;
            }

            let is_selected = self
                .selected_path
                .as_ref()
                .is_some_and(|selected| *selected == entry.path);

            if is_selected {
                let row_rect = Rect::new(
                    sidebar_rect.x,
                    entry_y - 2.0,
                    sidebar_rect.width,
                    ROW_HEIGHT,
                );
                Panel::new(row_rect, ctx.theme.accent_dim).paint(ctx.scene);
            }

            let indent = sidebar_rect.x + 12.0 + entry.depth as f32 * 16.0;
            let (prefix, color) = if entry.is_directory {
                let chevron = if self.collapsed_dirs.contains(&entry.path) {
                    "\u{25b8} "
                } else {
                    "\u{25be} "
                };
                (chevron, ctx.theme.text_secondary)
            } else {
                ("\u{00b7} ", ctx.theme.text_primary)
            };

            let display_name = if !entry.is_directory {
                entry.name.strip_suffix(".md").unwrap_or(&entry.name)
            } else {
                &entry.name
            };
            let label = format!("{prefix}{display_name}");

            let max_width = sidebar_rect.x + sidebar_rect.width - indent - 8.0;
            let truncated = truncate_to_width(&label, max_width, ctx.theme.typography.small_size);

            draw_text(
                ctx.scene,
                ctx.text,
                &truncated,
                ctx.theme.typography.small_size,
                (indent, entry_y),
                color,
            );

            let row_rect = Rect::new(
                sidebar_rect.x,
                entry_y - 2.0,
                sidebar_rect.width,
                ROW_HEIGHT,
            );
            hits.push(HitId(FILE_ENTRY_HIT_BASE + index as u32), row_rect);

            entry_y += ROW_HEIGHT;
        }

        Panel::new(separator_rect, ctx.theme.separator).paint(ctx.scene);

        if !self.content_lines.is_empty() {
            let padding_left = 12.0;
            let padding_top = 16.0;
            let line_height =
                ctx.theme.typography.body_size * ctx.theme.typography.line_height_factor;
            let mut line_y = content_rect.y + padding_top;

            for line in &self.content_lines {
                if line_y > content_rect.y + content_rect.height {
                    break;
                }
                draw_text(
                    ctx.scene,
                    ctx.text,
                    line,
                    ctx.theme.typography.body_size,
                    (content_rect.x + padding_left, line_y),
                    ctx.theme.text_primary,
                );
                line_y += line_height;
            }
        }

        Ok(())
    }
}

/// Estimates character count that fits within `max_width` at `font_size`, appending ellipsis if truncated.
fn truncate_to_width(text: &str, max_width: f32, font_size: f32) -> String {
    let avg_char_width = font_size * 0.55;
    let max_chars = (max_width / avg_char_width).floor() as usize;
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{truncated}\u{2026}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_file_hit_above_base() {
        assert!(EditorView::is_file_hit(HitId(1000)));
        assert!(EditorView::is_file_hit(HitId(1500)));
    }

    #[test]
    fn is_file_hit_below_base() {
        assert!(!EditorView::is_file_hit(HitId(0)));
        assert!(!EditorView::is_file_hit(HitId(999)));
    }

    #[test]
    fn handle_click_file_populates_content() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("test.md"), "line one\nline two").unwrap();

        let vault = Vault::open(root).unwrap();
        let mut editor = EditorView::new(&vault);

        let flat = flatten_tree_filtered(&editor.file_tree, &editor.collapsed_dirs);
        let file_index = flat
            .iter()
            .position(|entry| entry.name == "test.md")
            .expect("test.md should be in tree");

        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + file_index as u32));

        assert_eq!(editor.content_lines, vec!["line one", "line two"]);
        assert_eq!(editor.selected_path, Some(root.join("test.md")));
    }

    #[test]
    fn handle_click_directory_toggles_collapsed() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join("notes")).unwrap();
        std::fs::write(root.join("notes/todo.md"), "# Todo").unwrap();

        let vault = Vault::open(root).unwrap();
        let mut editor = EditorView::new(&vault);

        let flat = flatten_tree_filtered(&editor.file_tree, &editor.collapsed_dirs);
        let dir_index = flat
            .iter()
            .position(|entry| entry.name == "notes")
            .expect("notes dir should be in tree");

        let hit = HitId(FILE_ENTRY_HIT_BASE + dir_index as u32);

        editor.handle_click(hit);
        assert!(editor.collapsed_dirs.contains(&root.join("notes")));

        editor.handle_click(hit);
        assert!(!editor.collapsed_dirs.contains(&root.join("notes")));
    }
}
