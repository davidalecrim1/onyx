use std::sync::Arc;
use cosmic_text::{Attrs, Buffer as TextBuffer, FontSystem, Metrics, SwashCache};
use vello::util::RenderContext;
use vello::{AaConfig, RenderParams, Renderer as VelloRenderer, RendererOptions, Scene};
use winit::window::Window;

pub struct Renderer {
    render_context: RenderContext,
    render_surface: vello::util::RenderSurface<'static>,
    vello: VelloRenderer,
    scene: Scene,
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl Renderer {
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

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.render_context
            .resize_surface(&mut self.render_surface, new_size.width, new_size.height);
    }

    pub fn render(&mut self) {
        let frame = match self.render_surface.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => return,
        };

        self.scene.reset();
        self.draw_text("Onyx — rendering foundation", 20.0, 40.0);

        let device_handle = &self.render_context.devices[self.render_surface.dev_id];

        self.vello
            .render_to_texture(
                &device_handle.device,
                &device_handle.queue,
                &self.scene,
                &self.render_surface.target_view,
                &RenderParams {
                    base_color: vello::peniko::Color::from_rgba8(26, 26, 30, 255),
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

    fn draw_text(&mut self, text: &str, x: f32, y: f32) {
        let metrics = Metrics::new(16.0, 20.0);
        let mut buffer = TextBuffer::new(&mut self.font_system, metrics);
        let width = self.render_surface.config.width as f32;
        buffer.set_size(&mut self.font_system, Some(width), None);
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new(),
            cosmic_text::Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((x, y), 1.0);
                if let Some(_image) =
                    self.swash_cache.get_image(&mut self.font_system, physical.cache_key)
                {
                }
            }
        }
    }
}
