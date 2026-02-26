use crate::text::{draw_text, measure_text};
use crate::ui::canvas::DrawContext;
use crate::ui::rect::Rect;
use vello::peniko::Color;

/// Horizontal alignment for label text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    Left,
    Center,
}

/// Builder for a single-line text label.
pub struct Label<'a> {
    text: &'a str,
    font_size: f32,
    color: Color,
    align: Align,
}

impl<'a> Label<'a> {
    /// Creates a label with explicit font size and color.
    pub fn new(text: &'a str, font_size: f32, color: Color) -> Self {
        Self {
            text,
            font_size,
            color,
            align: Align::Left,
        }
    }

    /// Sets horizontal alignment within the bounds.
    pub fn align(mut self, value: Align) -> Self {
        self.align = value;
        self
    }

    /// Measures and draws the label inside the given bounds.
    pub fn paint(self, ctx: &mut DrawContext, bounds: Rect) {
        let line_height = self.font_size * ctx.theme.typography.line_height_factor;
        let cap_height = self.font_size * 0.7;

        let label_x = match self.align {
            Align::Left => bounds.x,
            Align::Center => {
                let metrics = measure_text(ctx.text, self.text, self.font_size);
                bounds.x + (bounds.width - metrics.width) / 2.0
            }
        };

        let label_y = bounds.y + bounds.height / 2.0 + cap_height / 2.0 - line_height;

        draw_text(
            ctx.scene,
            ctx.text,
            self.text,
            self.font_size,
            (label_x, label_y),
            self.color,
        );
    }
}
