use std::collections::HashSet;
use std::path::PathBuf;

use taffy::style_helpers::{length, TaffyMaxContent};
use taffy::{FlexDirection, Size, Style, TaffyTree};

use crate::file_tree::{flatten_tree_filtered, scan_file_tree, FileTreeEntry};
use crate::text::draw_text;
use crate::ui::{DrawContext, HitId, HitSink, Panel, Rect};
use crate::vault::Vault;

const SIDEBAR_WIDTH: f32 = 240.0;
const FILE_ENTRY_HIT_BASE: u32 = 1000;
const TAB_HIT_BASE: u32 = 2000;
const TAB_CLOSE_HIT_BASE: u32 = 3000;
const ROW_HEIGHT: f32 = 28.0;
const TAB_BAR_HEIGHT: f32 = 36.0;
const TAB_PADDING_H: f32 = 12.0;
const TAB_CLOSE_SIZE: f32 = 16.0;
const SIDEBAR_PADDING_LEFT: f32 = 12.0;
const INDENT_PER_DEPTH: f32 = 20.0;
const HEADER_FONT_SIZE: f32 = 12.0;
const HEADER_HEIGHT: f32 = 32.0;

/// Single open file with its loaded content.
struct Tab {
    path: PathBuf,
    name: String,
    content_lines: Vec<String>,
}

/// Editor view with a file-tree sidebar, tab bar, and content area.
pub struct EditorView {
    vault_name: String,
    file_tree: Vec<FileTreeEntry>,
    tabs: Vec<Tab>,
    active_tab_index: Option<usize>,
    collapsed_dirs: HashSet<PathBuf>,
}

impl EditorView {
    /// Scans the vault's file tree on construction.
    pub fn new(vault: &Vault) -> Self {
        let file_tree = scan_file_tree(&vault.root).unwrap_or_default();
        Self {
            vault_name: vault.config.name.clone(),
            file_tree,
            tabs: Vec::new(),
            active_tab_index: None,
            collapsed_dirs: HashSet::new(),
        }
    }

    /// Returns true if the hit id belongs to a file tree entry.
    pub fn is_file_hit(id: HitId) -> bool {
        id.0 >= FILE_ENTRY_HIT_BASE && id.0 < TAB_HIT_BASE
    }

    /// Returns true if the hit id belongs to a tab label.
    pub fn is_tab_hit(id: HitId) -> bool {
        id.0 >= TAB_HIT_BASE && id.0 < TAB_CLOSE_HIT_BASE
    }

    /// Returns true if the hit id belongs to a tab close button.
    pub fn is_tab_close_hit(id: HitId) -> bool {
        id.0 >= TAB_CLOSE_HIT_BASE
    }

    /// Returns the path of the currently active tab, if any.
    fn active_path(&self) -> Option<&PathBuf> {
        self.active_tab_index
            .and_then(|index| self.tabs.get(index))
            .map(|tab| &tab.path)
    }

    /// Handles a click on a file tree entry, opening or focusing a tab.
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
            if let Some(existing) = self.tabs.iter().position(|tab| tab.path == path) {
                self.active_tab_index = Some(existing);
            } else {
                let name = entry.name.clone();
                let content_lines = load_file_content(&path);
                self.tabs.push(Tab {
                    path,
                    name,
                    content_lines,
                });
                self.active_tab_index = Some(self.tabs.len() - 1);
            }
        }
    }

    /// Switches the active tab.
    pub fn handle_tab_click(&mut self, hit_id: HitId) {
        let index = (hit_id.0 - TAB_HIT_BASE) as usize;
        if index < self.tabs.len() {
            self.active_tab_index = Some(index);
        }
    }

    /// Closes a tab and adjusts the active index.
    pub fn handle_tab_close(&mut self, hit_id: HitId) {
        let index = (hit_id.0 - TAB_CLOSE_HIT_BASE) as usize;
        if index >= self.tabs.len() {
            return;
        }

        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.active_tab_index = None;
        } else if let Some(active) = self.active_tab_index {
            if index == active {
                self.active_tab_index = Some(index.min(self.tabs.len() - 1));
            } else if index < active {
                self.active_tab_index = Some(active - 1);
            }
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

        let header_label = self.vault_name.to_uppercase();
        let header_text_y = sidebar_rect.y + (HEADER_HEIGHT - HEADER_FONT_SIZE) / 2.0;
        draw_text(
            ctx.scene,
            ctx.text,
            &header_label,
            HEADER_FONT_SIZE,
            (sidebar_rect.x + SIDEBAR_PADDING_LEFT, header_text_y),
            ctx.theme.text_secondary,
        );

        let flat = flatten_tree_filtered(&self.file_tree, &self.collapsed_dirs);
        let mut entry_y = sidebar_rect.y + HEADER_HEIGHT;
        for (index, entry) in flat.iter().enumerate() {
            if entry_y > sidebar_rect.y + sidebar_rect.height {
                break;
            }

            let row_rect = Rect::new(sidebar_rect.x, entry_y, sidebar_rect.width, ROW_HEIGHT);

            let is_selected = self
                .active_path()
                .is_some_and(|selected| *selected == entry.path);
            let is_hovered = row_rect.contains(ctx.cursor_position.0, ctx.cursor_position.1);

            if is_selected {
                Panel::new(row_rect, ctx.theme.surface_active).paint(ctx.scene);
            } else if is_hovered {
                Panel::new(row_rect, ctx.theme.surface_hover).paint(ctx.scene);
            }

            let indent =
                sidebar_rect.x + SIDEBAR_PADDING_LEFT + entry.depth as f32 * INDENT_PER_DEPTH;
            let text_y = entry_y + (ROW_HEIGHT - ctx.theme.typography.small_size) / 2.0;

            if entry.is_directory {
                let chevron = if self.collapsed_dirs.contains(&entry.path) {
                    "\u{25b8}"
                } else {
                    "\u{25be}"
                };
                draw_text(
                    ctx.scene,
                    ctx.text,
                    chevron,
                    ctx.theme.typography.small_size,
                    (indent, text_y),
                    ctx.theme.text_secondary,
                );

                let name_x = indent + ctx.theme.typography.small_size;
                let max_width = sidebar_rect.x + sidebar_rect.width - name_x - 8.0;
                let truncated =
                    truncate_to_width(&entry.name, max_width, ctx.theme.typography.small_size);
                draw_text(
                    ctx.scene,
                    ctx.text,
                    &truncated,
                    ctx.theme.typography.small_size,
                    (name_x, text_y),
                    ctx.theme.text_secondary,
                );
            } else {
                let display_name = entry.name.strip_suffix(".md").unwrap_or(&entry.name);
                let max_width = sidebar_rect.x + sidebar_rect.width - indent - 8.0;
                let truncated =
                    truncate_to_width(display_name, max_width, ctx.theme.typography.small_size);
                draw_text(
                    ctx.scene,
                    ctx.text,
                    &truncated,
                    ctx.theme.typography.small_size,
                    (indent, text_y),
                    ctx.theme.text_primary,
                );
            }

            hits.push(HitId(FILE_ENTRY_HIT_BASE + index as u32), row_rect);

            entry_y += ROW_HEIGHT;
        }

        Panel::new(separator_rect, ctx.theme.separator).paint(ctx.scene);

        let mut content_top = content_rect.y;

        if !self.tabs.is_empty() {
            let tab_bar_rect = Rect::new(
                content_rect.x,
                content_rect.y,
                content_rect.width,
                TAB_BAR_HEIGHT,
            );
            Panel::new(tab_bar_rect, ctx.theme.surface).paint(ctx.scene);

            let mut tab_x = content_rect.x;
            for (index, tab) in self.tabs.iter().enumerate() {
                let display_name = tab.name.strip_suffix(".md").unwrap_or(&tab.name);
                let label_width =
                    display_name.len() as f32 * ctx.theme.typography.small_size * 0.55;
                let tab_width = TAB_PADDING_H + label_width + TAB_PADDING_H + TAB_CLOSE_SIZE + 4.0;

                let is_active = self.active_tab_index == Some(index);
                let tab_rect = Rect::new(tab_x, tab_bar_rect.y, tab_width, TAB_BAR_HEIGHT);
                let is_tab_hovered =
                    tab_rect.contains(ctx.cursor_position.0, ctx.cursor_position.1);

                let background = if is_active {
                    ctx.theme.background
                } else if is_tab_hovered {
                    ctx.theme.surface_hover
                } else {
                    ctx.theme.surface
                };
                Panel::new(tab_rect, background).paint(ctx.scene);

                let text_y =
                    tab_bar_rect.y + (TAB_BAR_HEIGHT - ctx.theme.typography.small_size) / 2.0;
                let text_color = if is_active {
                    ctx.theme.text_primary
                } else {
                    ctx.theme.text_secondary
                };
                draw_text(
                    ctx.scene,
                    ctx.text,
                    display_name,
                    ctx.theme.typography.small_size,
                    (tab_x + TAB_PADDING_H, text_y),
                    text_color,
                );

                hits.push(HitId(TAB_HIT_BASE + index as u32), tab_rect);

                let close_x = tab_x + tab_width - TAB_CLOSE_SIZE - 4.0;
                let close_y = tab_bar_rect.y + (TAB_BAR_HEIGHT - TAB_CLOSE_SIZE) / 2.0;
                let close_rect = Rect::new(close_x, close_y, TAB_CLOSE_SIZE, TAB_CLOSE_SIZE);
                draw_text(
                    ctx.scene,
                    ctx.text,
                    "\u{00d7}",
                    ctx.theme.typography.small_size,
                    (close_x + 2.0, close_y),
                    ctx.theme.text_secondary,
                );
                hits.push(HitId(TAB_CLOSE_HIT_BASE + index as u32), close_rect);

                tab_x += tab_width;
            }

            let separator = Rect::new(
                content_rect.x,
                tab_bar_rect.y + TAB_BAR_HEIGHT - 1.0,
                content_rect.width,
                1.0,
            );
            Panel::new(separator, ctx.theme.border).paint(ctx.scene);

            content_top += TAB_BAR_HEIGHT;
        }

        if let Some(active_tab) = self.active_tab_index.and_then(|index| self.tabs.get(index)) {
            let padding_left = 16.0;
            let padding_top = 20.0;
            let line_height =
                ctx.theme.typography.body_size * ctx.theme.typography.line_height_factor;
            let mut line_y = content_top + padding_top;

            for line in &active_tab.content_lines {
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

/// Reads a file from disk and returns its lines, with fallbacks for binary and IO errors.
fn load_file_content(path: &PathBuf) -> Vec<String> {
    match std::fs::read(path) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => text.lines().map(String::from).collect(),
            Err(_) => vec!["Binary file \u{2014} cannot display".to_string()],
        },
        Err(error) => vec![format!("Error reading file: {error}")],
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
    fn is_file_hit_recognises_file_range() {
        assert!(EditorView::is_file_hit(HitId(1000)));
        assert!(EditorView::is_file_hit(HitId(1500)));
        assert!(EditorView::is_file_hit(HitId(1999)));
    }

    #[test]
    fn is_file_hit_rejects_outside_range() {
        assert!(!EditorView::is_file_hit(HitId(0)));
        assert!(!EditorView::is_file_hit(HitId(999)));
        assert!(!EditorView::is_file_hit(HitId(2000)));
        assert!(!EditorView::is_file_hit(HitId(3000)));
    }

    #[test]
    fn is_tab_hit_recognises_tab_range() {
        assert!(EditorView::is_tab_hit(HitId(2000)));
        assert!(EditorView::is_tab_hit(HitId(2500)));
        assert!(!EditorView::is_tab_hit(HitId(1999)));
        assert!(!EditorView::is_tab_hit(HitId(3000)));
    }

    #[test]
    fn is_tab_close_hit_recognises_close_range() {
        assert!(EditorView::is_tab_close_hit(HitId(3000)));
        assert!(EditorView::is_tab_close_hit(HitId(3500)));
        assert!(!EditorView::is_tab_close_hit(HitId(2999)));
    }

    #[test]
    fn handle_click_file_opens_tab() {
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

        assert_eq!(editor.tabs.len(), 1);
        assert_eq!(editor.active_tab_index, Some(0));
        assert_eq!(editor.tabs[0].content_lines, vec!["line one", "line two"]);
        assert_eq!(editor.tabs[0].path, root.join("test.md"));
    }

    #[test]
    fn handle_click_same_file_focuses_existing_tab() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("test.md"), "content").unwrap();

        let vault = Vault::open(root).unwrap();
        let mut editor = EditorView::new(&vault);

        let flat = flatten_tree_filtered(&editor.file_tree, &editor.collapsed_dirs);
        let file_index = flat
            .iter()
            .position(|entry| entry.name == "test.md")
            .expect("test.md should be in tree");
        let hit = HitId(FILE_ENTRY_HIT_BASE + file_index as u32);

        editor.handle_click(hit);
        editor.handle_click(hit);

        assert_eq!(editor.tabs.len(), 1);
        assert_eq!(editor.active_tab_index, Some(0));
    }

    #[test]
    fn handle_click_opens_multiple_tabs() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("a.md"), "alpha").unwrap();
        std::fs::write(root.join("b.md"), "beta").unwrap();

        let vault = Vault::open(root).unwrap();
        let mut editor = EditorView::new(&vault);

        let flat = flatten_tree_filtered(&editor.file_tree, &editor.collapsed_dirs);
        let index_a = flat.iter().position(|entry| entry.name == "a.md").unwrap();
        let index_b = flat.iter().position(|entry| entry.name == "b.md").unwrap();

        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_a as u32));
        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_b as u32));

        assert_eq!(editor.tabs.len(), 2);
        assert_eq!(editor.active_tab_index, Some(1));
    }

    #[test]
    fn handle_tab_click_switches_active() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("a.md"), "alpha").unwrap();
        std::fs::write(root.join("b.md"), "beta").unwrap();

        let vault = Vault::open(root).unwrap();
        let mut editor = EditorView::new(&vault);

        let flat = flatten_tree_filtered(&editor.file_tree, &editor.collapsed_dirs);
        let index_a = flat.iter().position(|entry| entry.name == "a.md").unwrap();
        let index_b = flat.iter().position(|entry| entry.name == "b.md").unwrap();

        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_a as u32));
        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_b as u32));
        assert_eq!(editor.active_tab_index, Some(1));

        editor.handle_tab_click(HitId(TAB_HIT_BASE));
        assert_eq!(editor.active_tab_index, Some(0));
    }

    #[test]
    fn handle_tab_close_removes_tab() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("a.md"), "alpha").unwrap();
        std::fs::write(root.join("b.md"), "beta").unwrap();

        let vault = Vault::open(root).unwrap();
        let mut editor = EditorView::new(&vault);

        let flat = flatten_tree_filtered(&editor.file_tree, &editor.collapsed_dirs);
        let index_a = flat.iter().position(|entry| entry.name == "a.md").unwrap();
        let index_b = flat.iter().position(|entry| entry.name == "b.md").unwrap();

        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_a as u32));
        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_b as u32));

        editor.handle_tab_close(HitId(TAB_CLOSE_HIT_BASE));
        assert_eq!(editor.tabs.len(), 1);
        assert_eq!(editor.tabs[0].name, "b.md");
        assert_eq!(editor.active_tab_index, Some(0));
    }

    #[test]
    fn handle_tab_close_last_tab_clears_active() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("test.md"), "content").unwrap();

        let vault = Vault::open(root).unwrap();
        let mut editor = EditorView::new(&vault);

        let flat = flatten_tree_filtered(&editor.file_tree, &editor.collapsed_dirs);
        let file_index = flat
            .iter()
            .position(|entry| entry.name == "test.md")
            .unwrap();

        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + file_index as u32));
        editor.handle_tab_close(HitId(TAB_CLOSE_HIT_BASE));

        assert!(editor.tabs.is_empty());
        assert_eq!(editor.active_tab_index, None);
    }

    #[test]
    fn handle_tab_close_adjusts_active_index_when_before() {
        let temp = tempfile::TempDir::new().unwrap();
        let root = temp.path();
        std::fs::write(root.join("a.md"), "alpha").unwrap();
        std::fs::write(root.join("b.md"), "beta").unwrap();
        std::fs::write(root.join("c.md"), "gamma").unwrap();

        let vault = Vault::open(root).unwrap();
        let mut editor = EditorView::new(&vault);

        let flat = flatten_tree_filtered(&editor.file_tree, &editor.collapsed_dirs);
        let index_a = flat.iter().position(|entry| entry.name == "a.md").unwrap();
        let index_b = flat.iter().position(|entry| entry.name == "b.md").unwrap();
        let index_c = flat.iter().position(|entry| entry.name == "c.md").unwrap();

        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_a as u32));
        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_b as u32));
        editor.handle_click(HitId(FILE_ENTRY_HIT_BASE + index_c as u32));
        assert_eq!(editor.active_tab_index, Some(2));

        // Close first tab; active (index 2) should shift to 1
        editor.handle_tab_close(HitId(TAB_CLOSE_HIT_BASE));
        assert_eq!(editor.active_tab_index, Some(1));
        assert_eq!(editor.tabs[1].name, "c.md");
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
