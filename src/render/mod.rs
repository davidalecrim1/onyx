pub mod terminal;
pub mod ui;

use std::sync::Arc;
use cosmic_text::{Attrs, Buffer as TextBuffer, FontSystem, Metrics, Style, SwashCache, Weight};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Blob, Brush, Color, Fill, ImageAlphaType, ImageData, ImageFormat};
use vello::util::RenderContext;
use vello::{AaConfig, RenderParams, Renderer as VelloRenderer, RendererOptions, Scene};
use winit::window::Window;

use crate::editor::{RenderLine, SpanStyle};

#[derive(Copy, Clone)]
pub(crate) enum CursorShape {
    Block,
    IBeam,
}

/// Returns the pixel x-coordinate of the cursor at `col` given per-glyph advance widths.
/// Falls back to col * fallback_advance if col exceeds the glyph count.
fn cursor_pixel_x(advances: &[f32], col: usize, left_pad: f32, fallback_advance: f32) -> f32 {
    let x: f32 = advances.iter().take(col).sum();
    left_pad + x + if col >= advances.len() {
        (col - advances.len()) as f32 * fallback_advance
    } else {
        0.0
    }
}

pub struct Renderer {
    render_context: RenderContext,
    render_surface: vello::util::RenderSurface<'static>,
    vello: VelloRenderer,
    pub scene: Scene,
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl Renderer {
    /// Initialises the full GPU pipeline; blocks until the async surface setup resolves.
    pub fn new(window: Arc<Window>) -> Self {
        pollster::block_on(Self::init(window))
    }

    async fn init(window: Arc<Window>) -> Self {
        let mut render_context = RenderContext::new();
        let size = window.inner_size();

        let render_surface = render_context
            .create_surface(
                window,
                size.width.max(1),
                size.height.max(1),
                wgpu::PresentMode::Fifo,
            )
            .await
            .expect("failed to create render surface");

        let device = &render_context.devices[render_surface.dev_id].device;
        let vello = VelloRenderer::new(device, RendererOptions::default())
            .expect("failed to create Vello renderer");
        let scene = Scene::new();
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        Renderer { render_context, render_surface, vello, scene, font_system, swash_cache }
    }

    /// Returns the current surface width in logical pixels.
    pub fn surface_width(&self) -> f32 {
        self.render_surface.config.width as f32
    }

    /// Returns the current surface height in logical pixels.
    pub fn surface_height(&self) -> f32 {
        self.render_surface.config.height as f32
    }

    /// Handles a window resize; skips zero-area sizes that would panic the surface.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.render_context
            .resize_surface(&mut self.render_surface, new_size.width, new_size.height);
    }

    /// Draws each buffer line and a cursor rectangle; kept for raw-mode fallback.
    #[allow(dead_code)]
    pub fn draw_buffer(&mut self, lines: &[String], cursor_line: usize, cursor_col: usize, scale_factor: f32) {
        let metrics = Metrics::new(15.0, 22.0);
        let line_height = 22.0_f32;
        let left_pad = 48.0_f32;
        let top_pad = 8.0_f32;
        let char_width = 9.0_f32; // approximate monospace advance width

        for (idx, line_text) in lines.iter().enumerate() {
            let y = top_pad + idx as f32 * line_height;

            if idx == cursor_line {
                let cx = left_pad + cursor_col as f32 * char_width;
                let cursor_rect = Rect::new(
                    cx as f64,
                    y as f64,
                    (cx + char_width) as f64,
                    (y + line_height) as f64,
                );
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(97, 175, 239, 180)),
                    None,
                    &cursor_rect,
                );
            }

            let mut text_buf = TextBuffer::new(&mut self.font_system, metrics);
            let surface_width = self.render_surface.config.width as f32;
            text_buf.set_size(&mut self.font_system, Some(surface_width), None);
            text_buf.set_text(
                &mut self.font_system,
                line_text,
                Attrs::new(),
                cosmic_text::Shaping::Advanced,
            );
            text_buf.shape_until_scroll(&mut self.font_system, false);

            let fg = Color::from_rgba8(220, 220, 220, 255);
            for run in text_buf.layout_runs() {
                for glyph in run.glyphs.iter() {
                    let physical = glyph.physical((left_pad, y), scale_factor);
                    self.blit_glyph(&physical, fg);
                }
            }
        }
    }

    /// Draws styled render lines from the editor layer and a cursor rectangle.
    pub fn draw_render_lines(
        &mut self,
        render_lines: &[RenderLine],
        cursor_line: usize,
        cursor_col: usize,
        cursor_shape: CursorShape,
        scale_factor: f32,
    ) {
        let left_pad = 48.0_f32;
        let top_pad = 8.0_f32;
        let base_line_height = 22.0_f32;
        let fallback_advance = 9.0_f32;
        let surface_width = self.render_surface.config.width as f32;

        for (line_idx, render_line) in render_lines.iter().enumerate() {
            let line_height = heading_line_height(&render_line.spans, base_line_height);
            let y = top_pad + line_idx as f32 * base_line_height;

            if render_line.spans.iter().any(|span| span.style == SpanStyle::CodeBlockText) {
                let bg = Rect::new(
                    left_pad as f64,
                    y as f64,
                    (surface_width - left_pad) as f64,
                    (y + line_height) as f64,
                );
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(30, 30, 36, 255)),
                    None,
                    &bg,
                );
            }

            let mut x = left_pad;
            let mut advances: Vec<f32> = Vec::new();
            for span in &render_line.spans {
                let font_size = span_font_size(&span.style);
                let metrics = Metrics::new(font_size, line_height);
                let mut text_buf = TextBuffer::new(&mut self.font_system, metrics);
                text_buf.set_size(&mut self.font_system, Some(surface_width - x), None);
                let attrs = span_attrs(&span.style);
                text_buf.set_text(
                    &mut self.font_system,
                    &span.text,
                    attrs,
                    cosmic_text::Shaping::Advanced,
                );
                text_buf.shape_until_scroll(&mut self.font_system, false);

                let fg = span_fg_color(&span.style);
                for run in text_buf.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical = glyph.physical((x, y), scale_factor);
                        self.blit_glyph(&physical, fg);
                        if line_idx == cursor_line {
                            advances.push(glyph.w);
                        }
                        x += glyph.w;
                    }
                }
            }

            if line_idx == cursor_line {
                let cx = cursor_pixel_x(&advances, cursor_col, left_pad, fallback_advance);
                let cursor_width = match cursor_shape {
                    CursorShape::Block => fallback_advance,
                    CursorShape::IBeam => 2.0,
                };
                let cursor_color = match cursor_shape {
                    CursorShape::Block => Color::from_rgba8(97, 175, 239, 180),
                    CursorShape::IBeam => Color::from_rgba8(97, 175, 239, 255),
                };
                let cursor_rect = Rect::new(
                    cx as f64,
                    y as f64,
                    (cx + cursor_width) as f64,
                    (y + line_height) as f64,
                );
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(cursor_color),
                    None,
                    &cursor_rect,
                );
            }
        }
    }

    /// Draws render lines with a vertical offset from the top of the surface.
    pub fn draw_render_lines_offset(
        &mut self,
        render_lines: &[RenderLine],
        cursor_line: usize,
        cursor_col: usize,
        cursor_shape: CursorShape,
        scroll_offset: usize,
        top_offset: f32,
        scale_factor: f32,
    ) {
        let left_pad = 48.0_f32;
        let top_pad = top_offset + 8.0_f32;
        let base_line_height = 22.0_f32;
        let fallback_advance = 9.0_f32;
        let surface_width = self.render_surface.config.width as f32;

        let visible = render_lines.get(scroll_offset..).unwrap_or(&[]);
        let cursor_line_local = cursor_line.saturating_sub(scroll_offset);

        for (line_idx, render_line) in visible.iter().enumerate() {
            let line_height = heading_line_height(&render_line.spans, base_line_height);
            let y = top_pad + line_idx as f32 * base_line_height;

            if render_line.spans.iter().any(|span| span.style == SpanStyle::CodeBlockText) {
                let bg = Rect::new(
                    left_pad as f64,
                    y as f64,
                    (surface_width - left_pad) as f64,
                    (y + line_height) as f64,
                );
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(Color::from_rgba8(30, 30, 36, 255)),
                    None,
                    &bg,
                );
            }

            let mut x = left_pad;
            let mut advances: Vec<f32> = Vec::new();
            for span in &render_line.spans {
                let font_size = span_font_size(&span.style);
                let metrics = Metrics::new(font_size, line_height);
                let mut text_buf = TextBuffer::new(&mut self.font_system, metrics);
                text_buf.set_size(&mut self.font_system, Some(surface_width - x), None);
                let attrs = span_attrs(&span.style);
                text_buf.set_text(
                    &mut self.font_system,
                    &span.text,
                    attrs,
                    cosmic_text::Shaping::Advanced,
                );
                text_buf.shape_until_scroll(&mut self.font_system, false);

                let fg = span_fg_color(&span.style);
                for run in text_buf.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical = glyph.physical((x, y), scale_factor);
                        self.blit_glyph(&physical, fg);
                        if line_idx == cursor_line_local {
                            advances.push(glyph.w);
                        }
                        x += glyph.w;
                    }
                }
            }

            if line_idx == cursor_line_local {
                let cx = cursor_pixel_x(&advances, cursor_col, left_pad, fallback_advance);
                let cursor_width = match cursor_shape {
                    CursorShape::Block => fallback_advance,
                    CursorShape::IBeam => 2.0,
                };
                let cursor_color = match cursor_shape {
                    CursorShape::Block => Color::from_rgba8(97, 175, 239, 180),
                    CursorShape::IBeam => Color::from_rgba8(97, 175, 239, 255),
                };
                let cursor_rect = Rect::new(
                    cx as f64,
                    y as f64,
                    (cx + cursor_width) as f64,
                    (y + line_height) as f64,
                );
                self.scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(cursor_color),
                    None,
                    &cursor_rect,
                );
            }
        }
    }

    /// Blits a single rasterized glyph into the scene at its physical screen position.
    fn blit_glyph(&mut self, physical: &cosmic_text::PhysicalGlyph, fg: Color) {
        let Some(swash_image) = self.swash_cache.get_image(&mut self.font_system, physical.cache_key) else {
            return;
        };
        let width = swash_image.placement.width;
        let height = swash_image.placement.height;
        if width == 0 || height == 0 {
            return;
        }
        let rgba = swash_to_rgba(swash_image, fg);
        let blob = Blob::new(std::sync::Arc::new(rgba));
        let image = ImageData {
            data: blob,
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width,
            height,
        };
        let glyph_x = (physical.x + swash_image.placement.left) as f64;
        let glyph_y = (physical.y - swash_image.placement.top) as f64;
        self.scene.draw_image(&image, Affine::translate((glyph_x, glyph_y)));
    }

    /// Submits the current scene to the GPU and presents the frame.
    pub fn render(&mut self) {
        let frame = match self.render_surface.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return,
        };

        let device_handle = &self.render_context.devices[self.render_surface.dev_id];

        self.vello
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &self.render_surface.target_view,
                &RenderParams {
                    base_color: Color::from_rgba8(26, 26, 30, 255),
                    width: self.render_surface.config.width,
                    height: self.render_surface.config.height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .expect("vello render failed");

        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device_handle
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.render_surface.blitter.copy(
            &device_handle.device,
            &mut encoder,
            &self.render_surface.target_view,
            &frame_view,
        );
        device_handle.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

fn heading_line_height(spans: &[crate::editor::RenderSpan], base: f32) -> f32 {
    match spans.first().map(|span| &span.style) {
        Some(SpanStyle::Heading(1)) => base * 2.0,
        Some(SpanStyle::Heading(2)) => base * 1.6,
        Some(SpanStyle::Heading(3)) => base * 1.3,
        _ => base,
    }
}

fn span_font_size(style: &SpanStyle) -> f32 {
    match style {
        SpanStyle::Heading(1) => 30.0,
        SpanStyle::Heading(2) => 24.0,
        SpanStyle::Heading(3) => 20.0,
        SpanStyle::Heading(_) => 16.0,
        SpanStyle::Code | SpanStyle::CodeBlockText => 14.0,
        _ => 15.0,
    }
}

/// Converts a rasterized swash glyph into a flat RGBA byte buffer.
///
/// Mask glyphs use the alpha channel from swash data and apply the foreground color to RGB.
/// Color glyphs pass through unchanged since they already carry RGBA data.
fn swash_to_rgba(image: &cosmic_text::SwashImage, fg: Color) -> Vec<u8> {
    let r = (fg.components[0] * 255.0) as u8;
    let g = (fg.components[1] * 255.0) as u8;
    let b = (fg.components[2] * 255.0) as u8;
    match image.content {
        cosmic_text::SwashContent::Mask | cosmic_text::SwashContent::SubpixelMask => {
            image.data.iter().flat_map(|&alpha| [r, g, b, alpha]).collect()
        }
        cosmic_text::SwashContent::Color => image.data.to_vec(),
    }
}

fn span_fg_color(style: &SpanStyle) -> Color {
    match style {
        SpanStyle::Code | SpanStyle::CodeBlockText => Color::from_rgba8(171, 200, 148, 255),
        SpanStyle::BulletMarker => Color::from_rgba8(97, 175, 239, 255),
        _ => Color::from_rgba8(220, 220, 220, 255),
    }
}

fn span_attrs(style: &SpanStyle) -> Attrs<'static> {
    match style {
        SpanStyle::Bold | SpanStyle::Heading(_) => Attrs::new().weight(Weight::BOLD),
        SpanStyle::Italic => Attrs::new().style(Style::Italic),
        _ => Attrs::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{cursor_pixel_x, swash_to_rgba};
    use cosmic_text::{SwashContent, SwashImage};
    use vello::peniko::Color;

    #[test]
    fn cursor_x_after_two_glyphs() {
        let advances = vec![8.0_f32, 8.0_f32];
        let result = cursor_pixel_x(&advances, 2, 48.0, 9.0);
        assert_eq!(result, 64.0); // 48.0 (left_pad) + 16.0 (8+8)
    }

    #[test]
    fn cursor_x_fallback_beyond_glyphs() {
        let advances = vec![8.0_f32];
        let result = cursor_pixel_x(&advances, 3, 48.0, 9.0);
        assert_eq!(result, 48.0 + 8.0 + 2.0 * 9.0); // left_pad + 8.0 + 2 * fallback
    }

    #[test]
    fn cursor_x_at_col_zero() {
        let advances = vec![8.0_f32, 8.0_f32];
        let result = cursor_pixel_x(&advances, 0, 48.0, 9.0);
        assert_eq!(result, 48.0);
    }

    fn make_image(data: Vec<u8>, content: SwashContent, width: u32, height: u32) -> SwashImage {
        SwashImage {
            source: Default::default(),
            content,
            placement: cosmic_text::Placement { left: 0, top: 0, width, height },
            data,
        }
    }

    #[test]
    fn mask_glyph_expands_to_rgba() {
        let image = make_image(vec![128, 255], SwashContent::Mask, 2, 1);
        let fg = Color::from_rgba8(255, 200, 0, 255);
        let result = swash_to_rgba(&image, fg);
        assert_eq!(result, vec![255, 200, 0, 128, 255, 200, 0, 255]);
    }

    #[test]
    fn color_glyph_passes_through() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let image = make_image(data.clone(), SwashContent::Color, 2, 1);
        let fg = Color::from_rgba8(255, 255, 255, 255);
        let result = swash_to_rgba(&image, fg);
        assert_eq!(result, data);
    }
}
