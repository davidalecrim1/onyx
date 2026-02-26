use crate::ui::rect::Rect;
use vello::kurbo::Affine;
use vello::peniko::{Brush, Color, Fill};
use vello::Scene;

/// Filled rectangle background.
pub struct Panel {
    bounds: Rect,
    color: Color,
}

impl Panel {
    /// Creates a panel that fills the given bounds with a solid color.
    pub fn new(bounds: Rect, color: Color) -> Self {
        Self { bounds, color }
    }

    /// Paints the panel to the scene.
    pub fn paint(self, scene: &mut Scene) {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(self.color),
            None,
            &self.bounds.to_kurbo(),
        );
    }
}
