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

    /// Draws the sidebar file tree and empty content area.
    pub fn render(&self, ctx: &mut DrawContext, width: f32, height: f32) {
        let viewport = Rect::new(0.0, 0.0, width, height);
        let split = viewport.split_vertical(SIDEBAR_WIDTH);

        Panel::new(split.left, ctx.theme.surface).paint(ctx.scene);

        let header_y = 16.0;
        draw_text(
            ctx.scene,
            ctx.text,
            &self.vault_name,
            20.0,
            (12.0, header_y),
            ctx.theme.text_primary,
        );

        let flat = flatten_tree(&self.file_tree);
        let mut entry_y = header_y + 36.0;
        for entry in flat {
            let indent = 12.0 + entry.depth as f32 * 16.0;
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

        Panel::new(
            Rect::new(SIDEBAR_WIDTH, 0.0, 1.0, height),
            ctx.theme.separator,
        )
        .paint(ctx.scene);
    }
}
