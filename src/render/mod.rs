pub mod ui;

use std::sync::Arc;
use cosmic_text::{Attrs, Buffer as TextBuffer, FontSystem, Metrics, Style, SwashCache, Weight};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Brush, Color, Fill};
use vello::util::RenderContext;
use vello::{AaConfig, RenderParams, Renderer as VelloRenderer, RendererOptions, Scene};
use winit::window::Window;

use crate::editor::{RenderLine, SpanStyle};

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
    pub fn draw_buffer(&mut self, lines: &[String], cursor_line: usize, cursor_col: usize) {
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

            for run in text_buf.layout_runs() {
                for glyph in run.glyphs.iter() {
                    let physical = glyph.physical((left_pad, y), 1.0);
                    // Rasterise via swash so the glyph cache is warm; full blit in Milestone 3.
                    let _ = self.swash_cache.get_image(&mut self.font_system, physical.cache_key);
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
    ) {
        let left_pad = 48.0_f32;
        let top_pad = 8.0_f32;
        let base_line_height = 22.0_f32;
        let char_width = 9.0_f32;
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

            if line_idx == cursor_line {
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

            let mut x = left_pad;
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

                for run in text_buf.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical = glyph.physical((x, y), 1.0);
                        // Rasterise via swash to warm the glyph cache; full blit in Milestone 5.
                        let _ = self.swash_cache.get_image(&mut self.font_system, physical.cache_key);
                        x += glyph.w;
                    }
                }
            }
        }
    }

    /// Draws render lines with a vertical offset from the top of the surface.
    pub fn draw_render_lines_offset(
        &mut self,
        render_lines: &[RenderLine],
        cursor_line: usize,
        cursor_col: usize,
        top_offset: f32,
    ) {
        let left_pad = 48.0_f32;
        let top_pad = top_offset + 8.0_f32;
        let base_line_height = 22.0_f32;
        let char_width = 9.0_f32;
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

            if line_idx == cursor_line {
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

            let mut x = left_pad;
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

                for run in text_buf.layout_runs() {
                    for glyph in run.glyphs.iter() {
                        let physical = glyph.physical((x, y), 1.0);
                        let _ = self.swash_cache.get_image(&mut self.font_system, physical.cache_key);
                        x += glyph.w;
                    }
                }
            }
        }
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

fn span_attrs(style: &SpanStyle) -> Attrs<'static> {
    match style {
        SpanStyle::Bold | SpanStyle::Heading(_) => Attrs::new().weight(Weight::BOLD),
        SpanStyle::Italic => Attrs::new().style(Style::Italic),
        _ => Attrs::new(),
    }
}
