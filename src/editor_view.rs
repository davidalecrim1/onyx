use crate::file_tree::{flatten_tree, scan_file_tree, FileTreeEntry};
use crate::text::{draw_text, TextSystem};
use crate::vault::Vault;
use vello::kurbo::Affine;
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

const SIDEBAR_WIDTH: f32 = 220.0;
const SIDEBAR_BG: Color = Color::from_rgb8(32, 32, 38);
const SEPARATOR: Color = Color::from_rgb8(50, 50, 60);
const TEXT_PRIMARY: Color = Color::from_rgb8(230, 230, 230);
const TEXT_SECONDARY: Color = Color::from_rgb8(160, 160, 170);

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
    pub fn render(
        &self,
        scene: &mut Scene,
        text_system: &mut TextSystem,
        _width: f32,
        height: f32,
    ) {
        // Sidebar background
        let sidebar_rect = vello::kurbo::Rect::new(0.0, 0.0, SIDEBAR_WIDTH as f64, height as f64);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(SIDEBAR_BG),
            None,
            &sidebar_rect,
        );

        // Vault name header
        let header_y = 16.0;
        draw_text(
            scene,
            text_system,
            &self.vault_name,
            20.0,
            (12.0, header_y),
            TEXT_PRIMARY,
        );

        // File entries
        let flat = flatten_tree(&self.file_tree);
        let mut entry_y = header_y + 36.0;
        for entry in flat {
            let indent = 12.0 + entry.depth as f32 * 16.0;
            let color = if entry.is_directory {
                TEXT_SECONDARY
            } else {
                TEXT_PRIMARY
            };
            draw_text(
                scene,
                text_system,
                &entry.name,
                16.0,
                (indent, entry_y),
                color,
            );
            entry_y += 22.0;
        }

        // Vertical separator
        let separator_rect = vello::kurbo::Rect::new(
            SIDEBAR_WIDTH as f64,
            0.0,
            (SIDEBAR_WIDTH + 1.0) as f64,
            height as f64,
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(SEPARATOR),
            None,
            &separator_rect,
        );
    }
}
