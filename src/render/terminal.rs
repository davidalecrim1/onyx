use crate::terminal::TerminalGrid;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

/// Rasterises the terminal cell grid (backgrounds + cursor block) into the given scene.
pub fn draw_terminal(
    scene: &mut Scene,
    grid: &TerminalGrid,
    origin_x: f32,
    origin_y: f32,
    cell_width: f32,
    cell_height: f32,
) {
    let pane_w = grid.cols as f32 * cell_width;
    let pane_h = grid.rows as f32 * cell_height;
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(Color::from_rgba8(20, 20, 24, 255)),
        None,
        &Rect::new(origin_x as f64, origin_y as f64, (origin_x + pane_w) as f64, (origin_y + pane_h) as f64),
    );

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            let cell = grid.cell(row, col);
            let x = origin_x + col as f32 * cell_width;
            let y = origin_y + row as f32 * cell_height;

            if cell.bg.r != 26 || cell.bg.g != 26 || cell.bg.b != 30 {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(cell.bg.r, cell.bg.g, cell.bg.b, 255)),
                    None,
                    &Rect::new(x as f64, y as f64, (x + cell_width) as f64, (y + cell_height) as f64),
                );
            }

            if row == grid.cursor_row && col == grid.cursor_col {
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(97, 175, 239, 200)),
                    None,
                    &Rect::new(x as f64, y as f64, (x + cell_width) as f64, (y + cell_height) as f64),
                );
            }

            // Character rendering via cosmic-text is wired in the app layer; the
            // colour rectangles here prove cell boundaries for the MVP.
            let _ = cell.ch;
        }
    }
}
