use taffy::style_helpers::{length, TaffyMaxContent};
use taffy::{FlexDirection, Size, Style, TaffyTree};

use crate::file_tree::{flatten_tree, scan_file_tree, FileTreeEntry};
use crate::text::draw_text;
use crate::ui::{DrawContext, Panel, Rect};
use crate::vault::Vault;

const SIDEBAR_WIDTH: f32 = 220.0;

/// Minimal editor view with a file-tree sidebar and empty content area.
pub struct EditorView {
    vault_name: String,
    file_tree: Vec<FileTreeEntry>,
}

impl EditorView {
    /// Scans the vault's file tree on construction.
    pub fn new(vault: &Vault) -> Self {
        let file_tree = scan_file_tree(&vault.root).unwrap_or_default();
        Self {
            vault_name: vault.config.name.clone(),
            file_tree,
        }
    }

    /// Draws the sidebar file tree and empty content area using Taffy for layout.
    pub fn render(&self, ctx: &mut DrawContext, bounds: Rect) -> Result<(), taffy::TaffyError> {
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

        let flat = flatten_tree(&self.file_tree);
        let mut entry_y = sidebar_rect.y + header_y + 36.0;
        for entry in flat {
            let indent = sidebar_rect.x + 12.0 + entry.depth as f32 * 16.0;
            let color = if entry.is_directory {
                ctx.theme.text_secondary
            } else {
                ctx.theme.text_primary
            };
            draw_text(
                ctx.scene,
                ctx.text,
                &entry.name,
                ctx.theme.typography.small_size,
                (indent, entry_y),
                color,
            );
            entry_y += 22.0;
        }

        Panel::new(separator_rect, ctx.theme.separator).paint(ctx.scene);

        Ok(())
    }
}
