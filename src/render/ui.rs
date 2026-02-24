use crate::shell::FileEntry;
use vello::kurbo::{Affine, Line, Rect, Stroke};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

pub const TAB_HEIGHT: f32 = 32.0;
pub const FILE_TREE_WIDTH: f32 = 220.0;

const DIVIDER_COLOR: Color = Color::from_rgba8(50, 50, 58, 255);
const TAB_BG: Color = Color::from_rgba8(30, 30, 36, 255);
const TAB_ACTIVE_BG: Color = Color::from_rgba8(40, 40, 48, 255);
const FILE_TREE_BG: Color = Color::from_rgba8(24, 24, 30, 255);

/// Draws the tab bar background, individual tab slots, and a bottom border line.
pub fn draw_tab_bar(scene: &mut Scene, tabs: &[String], active: usize, width: f32) {
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(TAB_BG),
        None,
        &Rect::new(0.0, 0.0, width as f64, TAB_HEIGHT as f64),
    );

    let tab_width = 120.0_f32;
    for (index, _name) in tabs.iter().enumerate() {
        let x = index as f32 * tab_width;
        let bg = if index == active { TAB_ACTIVE_BG } else { TAB_BG };
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(bg),
            None,
            &Rect::new(x as f64, 0.0, (x + tab_width) as f64, TAB_HEIGHT as f64),
        );
    }

    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(DIVIDER_COLOR),
        None,
        &Line::new((0.0, TAB_HEIGHT as f64), (width as f64, TAB_HEIGHT as f64)),
    );
}

/// Draws the file tree panel background, a right border, and a highlight row for the selected entry.
pub fn draw_file_tree(
    scene: &mut Scene,
    entries: &[FileEntry],
    selected: Option<usize>,
    height: f32,
) {
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(FILE_TREE_BG),
        None,
        &Rect::new(0.0, TAB_HEIGHT as f64, FILE_TREE_WIDTH as f64, height as f64),
    );

    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(DIVIDER_COLOR),
        None,
        &Line::new(
            (FILE_TREE_WIDTH as f64, TAB_HEIGHT as f64),
            (FILE_TREE_WIDTH as f64, height as f64),
        ),
    );

    let row_height = 22.0_f32;
    for (index, _entry) in entries.iter().enumerate() {
        if selected == Some(index) {
            let y = TAB_HEIGHT + index as f32 * row_height;
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(50, 100, 180, 80)),
                None,
                &Rect::new(0.0, y as f64, FILE_TREE_WIDTH as f64, (y + row_height) as f64),
            );
        }
    }
}
