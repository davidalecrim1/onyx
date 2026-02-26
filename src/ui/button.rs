use crate::text::{draw_text, measure_text};
use crate::ui::canvas::DrawContext;
use crate::ui::hit_test::{HitId, HitSink};
use crate::ui::rect::Rect;
use vello::kurbo::Affine;
use vello::peniko::{Brush, Fill};

/// Builder for a rounded-rect button with centered label.
pub struct Button<'a> {
    label: &'a str,
    bounds: Rect,
    accent: bool,
    hit_id: Option<HitId>,
}

impl<'a> Button<'a> {
    /// Creates a button with the given label and position.
    pub fn new(label: &'a str, bounds: Rect) -> Self {
        Self {
            label,
            bounds,
            accent: false,
            hit_id: None,
        }
    }

    /// Uses the accent color instead of the dim variant.
    pub fn accent(mut self, value: bool) -> Self {
        self.accent = value;
        self
    }

    /// Registers this button as a clickable region.
    pub fn hit_id(mut self, id: HitId) -> Self {
        self.hit_id = Some(id);
        self
    }

    /// Paints the button and registers its hit region.
    pub fn paint(self, ctx: &mut DrawContext, hits: &mut HitSink) {
        let fill_color = if self.accent {
            ctx.theme.accent
        } else {
            ctx.theme.accent_dim
        };

        let rounded = self.bounds.to_rounded(5.0);
        ctx.scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill_color),
            None,
            &rounded,
        );

        let font_size = ctx.theme.typography.body_size;
        let line_height = font_size * ctx.theme.typography.line_height_factor;
        let cap_height = font_size * 0.7;
        let label_metrics = measure_text(ctx.text, self.label, font_size);
        let label_x = self.bounds.x + (self.bounds.width - label_metrics.width) / 2.0;
        let label_y = self.bounds.y + self.bounds.height / 2.0 + cap_height / 2.0 - line_height;

        draw_text(
            ctx.scene,
            ctx.text,
            self.label,
            font_size,
            (label_x, label_y),
            ctx.theme.text_primary,
        );

        if let Some(id) = self.hit_id {
            hits.push(id, self.bounds);
        }
    }
}
