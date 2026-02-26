use std::sync::Arc;

use vello::peniko::Color;
use vello::util::{RenderContext, RenderSurface};
use vello::wgpu;
use vello::{AaConfig, RenderParams, Renderer, RendererOptions, Scene};

use crate::error::OnyxError;

pub const BACKGROUND: Color = Color::from_rgb8(28, 28, 32);

/// Tracks whether the wgpu surface is currently attached to a window.
#[allow(dead_code)]
enum SurfaceState<'window> {
    Suspended,
    Active {
        window: Arc<winit::window::Window>,
        surface: Box<RenderSurface<'window>>,
    },
}

/// Owns the wgpu device, vello renderer, and the current surface.
pub struct GpuRenderer<'window> {
    render_context: RenderContext,
    renderer: Option<Renderer>,
    state: SurfaceState<'window>,
}

impl<'window> Default for GpuRenderer<'window> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'window> GpuRenderer<'window> {
    /// Creates a renderer in suspended state — call `resume` once a window is available.
    pub fn new() -> Self {
        Self {
            render_context: RenderContext::new(),
            renderer: None,
            state: SurfaceState::Suspended,
        }
    }

    /// Attaches to a window and creates the wgpu surface.
    pub fn resume(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Result<Arc<winit::window::Window>, OnyxError> {
        let attributes = winit::window::Window::default_attributes()
            .with_title("Onyx")
            .with_inner_size(winit::dpi::LogicalSize::new(900, 600));

        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|error: winit::error::OsError| OnyxError::Surface(error.to_string()))?,
        );

        let size = window.inner_size();
        let surface = pollster::block_on(self.render_context.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        ))
        .map_err(|error| OnyxError::Surface(error.to_string()))?;

        let device = &self.render_context.devices[surface.dev_id];
        let renderer = Renderer::new(
            &device.device,
            RendererOptions {
                antialiasing_support: vello::AaSupport::area_only(),
                ..Default::default()
            },
        )
        .map_err(|error| OnyxError::Renderer(error.to_string()))?;

        self.renderer = Some(renderer);
        self.state = SurfaceState::Active {
            window: window.clone(),
            surface: Box::new(surface),
        };

        Ok(window)
    }

    /// Handles window resize by updating the surface configuration.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if let SurfaceState::Active {
            ref mut surface, ..
        } = self.state
        {
            self.render_context.resize_surface(surface, width, height);
        }
    }

    /// Renders a scene to the current surface.
    pub fn render(&mut self, scene: &Scene) -> Result<(), OnyxError> {
        let SurfaceState::Active {
            ref mut surface, ..
        } = self.state
        else {
            return Ok(());
        };

        let Some(ref mut renderer) = self.renderer else {
            return Ok(());
        };

        let device = &self.render_context.devices[surface.dev_id];
        let surface_texture = surface
            .surface
            .get_current_texture()
            .map_err(|error: wgpu::SurfaceError| OnyxError::Surface(error.to_string()))?;

        renderer
            .render_to_texture(
                &device.device,
                &device.queue,
                scene,
                &surface.target_view,
                &RenderParams {
                    base_color: BACKGROUND,
                    width: surface.config.width,
                    height: surface.config.height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .map_err(|error| OnyxError::Renderer(error.to_string()))?;

        let mut encoder = device
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("blit"),
            });

        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        surface.blitter.copy(
            &device.device,
            &mut encoder,
            &surface.target_view,
            &target_view,
        );
        device.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }

    /// Drops the surface when the window is suspended.
    pub fn suspend(&mut self) {
        self.state = SurfaceState::Suspended;
    }
}
