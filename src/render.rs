use std::sync::Arc;
use cosmic_text::{Attrs, Buffer as TextBuffer, FontSystem, Metrics, SwashCache};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Brush, Color, Fill};
use vello::util::RenderContext;
use vello::{AaConfig, RenderParams, Renderer as VelloRenderer, RendererOptions, Scene};
use winit::window::Window;

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

    /// Handles a window resize; skips zero-area sizes that would panic the surface.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.render_context
            .resize_surface(&mut self.render_surface, new_size.width, new_size.height);
    }

    /// Draws each buffer line and a cursor rectangle, then submits the scene to the GPU.
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
