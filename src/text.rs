use std::sync::Arc;

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use vello::kurbo::Affine;
use vello::peniko::{Blob, Brush, Color, FontData};
use vello::{Glyph, Scene};

/// Width and height of a laid-out text run.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
}

/// Caches font data shared between cosmic-text shaping and vello rendering.
#[allow(dead_code)]
pub struct TextSystem {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    font_data_cache: Vec<(cosmic_text::fontdb::ID, FontData)>,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    /// Initialises with system fonts.
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
            font_data_cache: Vec::new(),
        }
    }

    fn get_vello_font(&mut self, font_id: cosmic_text::fontdb::ID) -> Option<FontData> {
        if let Some((_, font)) = self.font_data_cache.iter().find(|(id, _)| *id == font_id) {
            return Some(font.clone());
        }

        let font_data = self
            .font_system
            .db()
            .with_face_data(font_id, |data, _| data.to_vec())?;

        let font = FontData::new(Blob::new(Arc::new(font_data)), 0);
        self.font_data_cache.push((font_id, font.clone()));
        Some(font)
    }
}

/// Draws a single line of text and returns its metrics.
pub fn draw_text(
    scene: &mut Scene,
    text_system: &mut TextSystem,
    text: &str,
    font_size: f32,
    position: (f32, f32),
    color: Color,
) -> TextMetrics {
    let metrics = Metrics::new(font_size, font_size * 1.2);
    let mut buffer = Buffer::new(&mut text_system.font_system, metrics);
    buffer.set_size(&mut text_system.font_system, Some(f32::MAX), Some(f32::MAX));
    buffer.set_text(
        &mut text_system.font_system,
        text,
        Attrs::new(),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(&mut text_system.font_system, false);

    let line_height = font_size * 1.2;
    let mut total_width: f32 = 0.0;

    for run in buffer.layout_runs() {
        let mut glyphs: Vec<(FontData, Glyph)> = Vec::new();

        for glyph in run.glyphs.iter() {
            let Some(vello_font) = text_system.get_vello_font(glyph.font_id) else {
                continue;
            };

            glyphs.push((
                vello_font,
                Glyph {
                    id: glyph.glyph_id as u32,
                    x: glyph.x,
                    y: 0.0,
                },
            ));

            let end = glyph.x + glyph.w;
            if end > total_width {
                total_width = end;
            }
        }

        // Group consecutive glyphs by font and draw each batch
        let mut current_font: Option<FontData> = None;
        let mut current_batch: Vec<Glyph> = Vec::new();

        for (font, glyph) in glyphs {
            let same_font = current_font
                .as_ref()
                .map(|current| current.data.data().as_ptr() == font.data.data().as_ptr())
                .unwrap_or(false);

            if same_font {
                current_batch.push(glyph);
            } else {
                flush_glyphs(
                    scene,
                    &current_font,
                    &current_batch,
                    font_size,
                    position,
                    line_height,
                    color,
                );
                current_font = Some(font);
                current_batch = vec![glyph];
            }
        }

        flush_glyphs(
            scene,
            &current_font,
            &current_batch,
            font_size,
            position,
            line_height,
            color,
        );
    }

    TextMetrics {
        width: total_width,
        height: line_height,
    }
}

fn flush_glyphs(
    scene: &mut Scene,
    font: &Option<FontData>,
    batch: &[Glyph],
    font_size: f32,
    position: (f32, f32),
    line_height: f32,
    color: Color,
) {
    let Some(ref font) = font else { return };
    if batch.is_empty() {
        return;
    }

    scene
        .draw_glyphs(font)
        .font_size(font_size)
        .transform(Affine::translate((
            position.0 as f64,
            position.1 as f64 + line_height as f64,
        )))
        .brush(&Brush::Solid(color))
        .draw(vello::peniko::Fill::NonZero, batch.iter().copied());
}

/// Measures text without drawing it.
pub fn measure_text(text_system: &mut TextSystem, text: &str, font_size: f32) -> TextMetrics {
    let metrics = Metrics::new(font_size, font_size * 1.2);
    let mut buffer = Buffer::new(&mut text_system.font_system, metrics);
    buffer.set_size(&mut text_system.font_system, Some(f32::MAX), Some(f32::MAX));
    buffer.set_text(
        &mut text_system.font_system,
        text,
        Attrs::new(),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(&mut text_system.font_system, false);

    let mut total_width: f32 = 0.0;
    for run in buffer.layout_runs() {
        for glyph in run.glyphs.iter() {
            let end = glyph.x + glyph.w;
            if end > total_width {
                total_width = end;
            }
        }
    }

    TextMetrics {
        width: total_width,
        height: font_size * 1.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_metrics_non_zero() {
        let mut text_system = TextSystem::new();
        let metrics = measure_text(&mut text_system, "Hello, world!", 16.0);

        assert!(metrics.width > 0.0);
        assert!(metrics.height > 0.0);
    }

    #[test]
    fn empty_text_has_zero_width() {
        let mut text_system = TextSystem::new();
        let metrics = measure_text(&mut text_system, "", 16.0);

        assert_eq!(metrics.width, 0.0);
        assert!(metrics.height > 0.0);
    }
}
