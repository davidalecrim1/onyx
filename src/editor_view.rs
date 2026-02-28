use std::collections::HashSet;
use std::path::PathBuf;

use taffy::style_helpers::{length, TaffyMaxContent};
use taffy::{FlexDirection, Size, Style, TaffyTree};

use crate::action::Action;
use crate::file_tree::{flatten_tree_filtered, scan_file_tree, FileTreeEntry};
use crate::text::{draw_text, measure_text};
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

const CONTENT_AREA_HIT: u32 = 4000;
const CONTENT_PADDING_LEFT: f32 = 16.0;
const CONTENT_PADDING_TOP: f32 = 20.0;

/// Single open file with its loaded content.
struct Tab {
    path: PathBuf,
    name: String,
    content_lines: Vec<String>,
    saved_content: Vec<String>,
    cursor_line: usize,
    cursor_column: usize,
}

impl Tab {
    /// Compares current content against the last-saved snapshot.
    fn is_dirty(&self) -> bool {
        self.content_lines != self.saved_content
    }
}

/// Editor view with a file-tree sidebar, tab bar, and content area.
pub struct EditorView {
    vault_name: String,
    file_tree: Vec<FileTreeEntry>,
    tabs: Vec<Tab>,
    active_tab_index: Option<usize>,
    collapsed_dirs: HashSet<PathBuf>,
    content_origin_x: f32,
    content_origin_y: f32,
    content_line_height: f32,
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
            content_origin_x: 0.0,
            content_origin_y: 0.0,
            content_line_height: 0.0,
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
                let saved_content = content_lines.clone();
                self.tabs.push(Tab {
                    path,
                    name,
                    content_lines,
                    saved_content,
                    cursor_line: 0,
                    cursor_column: 0,
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

    /// Returns true if the hit id belongs to the content area.
    pub fn is_content_hit(id: HitId) -> bool {
        id.0 == CONTENT_AREA_HIT
    }

    /// Dispatches an action to the active tab's editing state.
    pub fn handle_action(&mut self, action: &Action) {
        let Some(index) = self.active_tab_index else {
            return;
        };
        let Some(tab) = self.tabs.get_mut(index) else {
            return;
        };

        match action {
            Action::InsertChar(ch) => insert_char(tab, *ch),
            Action::Backspace => backspace(tab),
            Action::Delete => delete_char(tab),
            Action::Enter => insert_newline(tab),
            Action::MoveLeft => move_left(tab),
            Action::MoveRight => move_right(tab),
            Action::MoveUp => move_up(tab),
            Action::MoveDown => move_down(tab),
            Action::MoveHome => tab.cursor_column = 0,
            Action::MoveEnd => {
                if let Some(line) = tab.content_lines.get(tab.cursor_line) {
                    tab.cursor_column = line.chars().count();
                }
            }
            Action::Save => save_tab(tab),
        }
    }

    /// Places the cursor at the clicked position in the content area.
    pub fn handle_content_click(
        &mut self,
        click_x: f32,
        click_y: f32,
        text: &mut crate::text::TextSystem,
        font_size: f32,
    ) {
        let Some(index) = self.active_tab_index else {
            return;
        };
        let Some(tab) = self.tabs.get_mut(index) else {
            return;
        };

        if self.content_line_height <= 0.0 {
            return;
        }

        let relative_y = click_y - self.content_origin_y;
        let line = (relative_y / self.content_line_height).floor() as usize;
        let line = line.min(tab.content_lines.len().saturating_sub(1));

        let relative_x = click_x - self.content_origin_x;
        let column = find_column_for_x(&tab.content_lines[line], relative_x, text, font_size);

        tab.cursor_line = line;
        tab.cursor_column = column;
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
        &mut self,
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

                let is_close_hovered =
                    close_rect.contains(ctx.cursor_position.0, ctx.cursor_position.1);
                let show_dirty_dot = tab.is_dirty() && !is_close_hovered;

                if show_dirty_dot {
                    draw_text(
                        ctx.scene,
                        ctx.text,
                        "\u{25CF}",
                        8.0,
                        (close_x + 4.0, close_y + 3.0),
                        ctx.theme.text_primary,
                    );
                } else {
                    draw_text(
                        ctx.scene,
                        ctx.text,
                        "\u{00d7}",
                        ctx.theme.typography.small_size,
                        (close_x + 2.0, close_y),
                        ctx.theme.text_secondary,
                    );
                }
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

        let content_area_rect = Rect::new(
            content_rect.x,
            content_top,
            content_rect.width,
            content_rect.height - (content_top - content_rect.y),
        );
        hits.push(HitId(CONTENT_AREA_HIT), content_area_rect);

        let line_height = ctx.theme.typography.body_size * ctx.theme.typography.line_height_factor;
        self.content_origin_x = content_rect.x + CONTENT_PADDING_LEFT;
        self.content_origin_y = content_top + CONTENT_PADDING_TOP;
        self.content_line_height = line_height;

        if let Some(active_tab) = self.active_tab_index.and_then(|index| self.tabs.get(index)) {
            let mut line_y = content_top + CONTENT_PADDING_TOP;

            for line in &active_tab.content_lines {
                if line_y > content_rect.y + content_rect.height {
                    break;
                }
                draw_text(
                    ctx.scene,
                    ctx.text,
                    line,
                    ctx.theme.typography.body_size,
                    (content_rect.x + CONTENT_PADDING_LEFT, line_y),
                    ctx.theme.text_primary,
                );
                line_y += line_height;
            }

            let cursor_line = active_tab.cursor_line;
            let cursor_column = active_tab.cursor_column;
            let cursor_y = self.content_origin_y + cursor_line as f32 * line_height;

            let cursor_x = if cursor_column > 0 {
                if let Some(current_line) = active_tab.content_lines.get(cursor_line) {
                    let prefix: String = current_line.chars().take(cursor_column).collect();
                    let metrics = measure_text(ctx.text, &prefix, ctx.theme.typography.body_size);
                    self.content_origin_x + metrics.width
                } else {
                    self.content_origin_x
                }
            } else {
                self.content_origin_x
            };

            let cursor_rect = Rect::new(cursor_x, cursor_y, 2.0, line_height);
            Panel::new(cursor_rect, ctx.theme.text_primary).paint(ctx.scene);
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

fn char_to_byte_index(line: &str, char_index: usize) -> usize {
    line.char_indices()
        .nth(char_index)
        .map(|(byte_pos, _)| byte_pos)
        .unwrap_or(line.len())
}

fn insert_char(tab: &mut Tab, ch: char) {
    if tab.content_lines.is_empty() {
        tab.content_lines.push(String::new());
        tab.saved_content = vec![];
    }
    let line = &mut tab.content_lines[tab.cursor_line];
    let byte_index = char_to_byte_index(line, tab.cursor_column);
    line.insert(byte_index, ch);
    tab.cursor_column += 1;
}

fn backspace(tab: &mut Tab) {
    if tab.content_lines.is_empty() {
        return;
    }

    if tab.cursor_column > 0 {
        let line = &mut tab.content_lines[tab.cursor_line];
        let byte_index = char_to_byte_index(line, tab.cursor_column - 1);
        let next_byte = char_to_byte_index(line, tab.cursor_column);
        line.drain(byte_index..next_byte);
        tab.cursor_column -= 1;
    } else if tab.cursor_line > 0 {
        let current_line = tab.content_lines.remove(tab.cursor_line);
        tab.cursor_line -= 1;
        let previous_char_count = tab.content_lines[tab.cursor_line].chars().count();
        tab.content_lines[tab.cursor_line].push_str(&current_line);
        tab.cursor_column = previous_char_count;
    }
}

fn delete_char(tab: &mut Tab) {
    if tab.content_lines.is_empty() {
        return;
    }

    let line_char_count = tab.content_lines[tab.cursor_line].chars().count();
    if tab.cursor_column < line_char_count {
        let line = &mut tab.content_lines[tab.cursor_line];
        let byte_index = char_to_byte_index(line, tab.cursor_column);
        let next_byte = char_to_byte_index(line, tab.cursor_column + 1);
        line.drain(byte_index..next_byte);
    } else if tab.cursor_line + 1 < tab.content_lines.len() {
        let next_line = tab.content_lines.remove(tab.cursor_line + 1);
        tab.content_lines[tab.cursor_line].push_str(&next_line);
    }
}

fn insert_newline(tab: &mut Tab) {
    if tab.content_lines.is_empty() {
        tab.content_lines.push(String::new());
        tab.content_lines.push(String::new());
        tab.cursor_line = 1;
        tab.cursor_column = 0;
        return;
    }

    let line = &tab.content_lines[tab.cursor_line];
    let byte_index = char_to_byte_index(line, tab.cursor_column);
    let remainder = line[byte_index..].to_string();
    tab.content_lines[tab.cursor_line].truncate(byte_index);
    tab.content_lines.insert(tab.cursor_line + 1, remainder);
    tab.cursor_line += 1;
    tab.cursor_column = 0;
}

fn move_left(tab: &mut Tab) {
    if tab.cursor_column > 0 {
        tab.cursor_column -= 1;
    } else if tab.cursor_line > 0 {
        tab.cursor_line -= 1;
        tab.cursor_column = tab.content_lines[tab.cursor_line].chars().count();
    }
}

fn move_right(tab: &mut Tab) {
    if tab.content_lines.is_empty() {
        return;
    }
    let line_len = tab.content_lines[tab.cursor_line].chars().count();
    if tab.cursor_column < line_len {
        tab.cursor_column += 1;
    } else if tab.cursor_line + 1 < tab.content_lines.len() {
        tab.cursor_line += 1;
        tab.cursor_column = 0;
    }
}

fn move_up(tab: &mut Tab) {
    if tab.cursor_line > 0 {
        tab.cursor_line -= 1;
        let line_len = tab.content_lines[tab.cursor_line].chars().count();
        tab.cursor_column = tab.cursor_column.min(line_len);
    }
}

fn move_down(tab: &mut Tab) {
    if tab.cursor_line + 1 < tab.content_lines.len() {
        tab.cursor_line += 1;
        let line_len = tab.content_lines[tab.cursor_line].chars().count();
        tab.cursor_column = tab.cursor_column.min(line_len);
    }
}

fn save_tab(tab: &mut Tab) {
    let content = tab.content_lines.join("\n");
    if let Err(error) = std::fs::write(&tab.path, &content) {
        log::error!("Failed to save {}: {error}", tab.path.display());
        return;
    }
    tab.saved_content = tab.content_lines.clone();
}

/// Finds the character column closest to a given x offset using binary search on measured widths.
fn find_column_for_x(
    line: &str,
    target_x: f32,
    text_system: &mut crate::text::TextSystem,
    font_size: f32,
) -> usize {
    let char_count = line.chars().count();
    if char_count == 0 || target_x <= 0.0 {
        return 0;
    }

    let mut low = 0usize;
    let mut high = char_count;

    while low < high {
        let mid = (low + high) / 2;
        let prefix: String = line.chars().take(mid).collect();
        let width = measure_text(text_system, &prefix, font_size).width;
        if width < target_x {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    // Check if click is closer to low-1 or low
    if low > 0 {
        let prev_prefix: String = line.chars().take(low - 1).collect();
        let prev_width = measure_text(text_system, &prev_prefix, font_size).width;
        let curr_prefix: String = line.chars().take(low).collect();
        let curr_width = measure_text(text_system, &curr_prefix, font_size).width;
        if (target_x - prev_width).abs() < (target_x - curr_width).abs() {
            return low - 1;
        }
    }

    low
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

    fn make_tab(lines: &[&str], cursor_line: usize, cursor_column: usize) -> Tab {
        let content_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        let saved_content = content_lines.clone();
        Tab {
            path: PathBuf::from("/tmp/test.md"),
            name: "test.md".to_string(),
            content_lines,
            saved_content,
            cursor_line,
            cursor_column,
        }
    }

    #[test]
    fn insert_char_at_start() {
        let mut tab = make_tab(&["hello"], 0, 0);
        insert_char(&mut tab, 'X');
        assert_eq!(tab.content_lines[0], "Xhello");
        assert_eq!(tab.cursor_column, 1);
    }

    #[test]
    fn insert_char_at_middle() {
        let mut tab = make_tab(&["hello"], 0, 2);
        insert_char(&mut tab, 'X');
        assert_eq!(tab.content_lines[0], "heXllo");
        assert_eq!(tab.cursor_column, 3);
    }

    #[test]
    fn insert_char_at_end() {
        let mut tab = make_tab(&["hello"], 0, 5);
        insert_char(&mut tab, '!');
        assert_eq!(tab.content_lines[0], "hello!");
        assert_eq!(tab.cursor_column, 6);
    }

    #[test]
    fn insert_char_empty_doc() {
        let mut tab = make_tab(&[], 0, 0);
        insert_char(&mut tab, 'A');
        assert_eq!(tab.content_lines, vec!["A"]);
        assert_eq!(tab.cursor_column, 1);
    }

    #[test]
    fn backspace_middle_of_line() {
        let mut tab = make_tab(&["hello"], 0, 3);
        backspace(&mut tab);
        assert_eq!(tab.content_lines[0], "helo");
        assert_eq!(tab.cursor_column, 2);
    }

    #[test]
    fn backspace_merges_lines() {
        let mut tab = make_tab(&["first", "second"], 1, 0);
        backspace(&mut tab);
        assert_eq!(tab.content_lines, vec!["firstsecond"]);
        assert_eq!(tab.cursor_line, 0);
        assert_eq!(tab.cursor_column, 5);
    }

    #[test]
    fn backspace_at_start_of_doc_is_noop() {
        let mut tab = make_tab(&["hello"], 0, 0);
        backspace(&mut tab);
        assert_eq!(tab.content_lines, vec!["hello"]);
        assert_eq!(tab.cursor_column, 0);
    }

    #[test]
    fn backspace_empty_doc_is_noop() {
        let mut tab = make_tab(&[], 0, 0);
        backspace(&mut tab);
        assert!(tab.content_lines.is_empty());
    }

    #[test]
    fn delete_char_middle_of_line() {
        let mut tab = make_tab(&["hello"], 0, 1);
        delete_char(&mut tab);
        assert_eq!(tab.content_lines[0], "hllo");
        assert_eq!(tab.cursor_column, 1);
    }

    #[test]
    fn delete_char_merges_next_line() {
        let mut tab = make_tab(&["first", "second"], 0, 5);
        delete_char(&mut tab);
        assert_eq!(tab.content_lines, vec!["firstsecond"]);
        assert_eq!(tab.cursor_line, 0);
    }

    #[test]
    fn insert_newline_splits_line() {
        let mut tab = make_tab(&["hello world"], 0, 5);
        insert_newline(&mut tab);
        assert_eq!(tab.content_lines, vec!["hello", " world"]);
        assert_eq!(tab.cursor_line, 1);
        assert_eq!(tab.cursor_column, 0);
    }

    #[test]
    fn insert_newline_at_end_of_line() {
        let mut tab = make_tab(&["hello"], 0, 5);
        insert_newline(&mut tab);
        assert_eq!(tab.content_lines, vec!["hello", ""]);
        assert_eq!(tab.cursor_line, 1);
        assert_eq!(tab.cursor_column, 0);
    }

    #[test]
    fn move_left_wraps_to_previous_line() {
        let mut tab = make_tab(&["abc", "def"], 1, 0);
        move_left(&mut tab);
        assert_eq!(tab.cursor_line, 0);
        assert_eq!(tab.cursor_column, 3);
    }

    #[test]
    fn move_right_wraps_to_next_line() {
        let mut tab = make_tab(&["abc", "def"], 0, 3);
        move_right(&mut tab);
        assert_eq!(tab.cursor_line, 1);
        assert_eq!(tab.cursor_column, 0);
    }

    #[test]
    fn move_up_clamps_column() {
        let mut tab = make_tab(&["ab", "longline"], 1, 7);
        move_up(&mut tab);
        assert_eq!(tab.cursor_line, 0);
        assert_eq!(tab.cursor_column, 2);
    }

    #[test]
    fn move_down_clamps_column() {
        let mut tab = make_tab(&["longline", "ab"], 0, 7);
        move_down(&mut tab);
        assert_eq!(tab.cursor_line, 1);
        assert_eq!(tab.cursor_column, 2);
    }

    #[test]
    fn is_dirty_after_edit() {
        let mut tab = make_tab(&["hello"], 0, 0);
        assert!(!tab.is_dirty());
        insert_char(&mut tab, 'X');
        assert!(tab.is_dirty());
    }

    #[test]
    fn save_clears_dirty() {
        let temp = tempfile::TempDir::new().unwrap();
        let file_path = temp.path().join("test.md");
        std::fs::write(&file_path, "hello").unwrap();

        let mut tab = make_tab(&["hello"], 0, 5);
        tab.path = file_path.clone();
        insert_char(&mut tab, '!');
        assert!(tab.is_dirty());

        save_tab(&mut tab);
        assert!(!tab.is_dirty());

        let saved = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(saved, "hello!");
    }

    #[test]
    fn is_content_hit_matches_content_area() {
        assert!(EditorView::is_content_hit(HitId(CONTENT_AREA_HIT)));
        assert!(!EditorView::is_content_hit(HitId(1000)));
        assert!(!EditorView::is_content_hit(HitId(3000)));
    }

    #[test]
    fn insert_char_utf8_multibyte() {
        let mut tab = make_tab(&["caf\u{00e9}"], 0, 4);
        insert_char(&mut tab, '!');
        assert_eq!(tab.content_lines[0], "caf\u{00e9}!");
        assert_eq!(tab.cursor_column, 5);
    }

    #[test]
    fn backspace_utf8_multibyte() {
        let mut tab = make_tab(&["caf\u{00e9}"], 0, 4);
        backspace(&mut tab);
        assert_eq!(tab.content_lines[0], "caf");
        assert_eq!(tab.cursor_column, 3);
    }
}
