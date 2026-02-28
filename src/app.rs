use std::sync::Arc;

use vello::Scene;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::{Window, WindowId};

use crate::editor_view::EditorView;
use crate::global_config::register_vault;
use crate::gpu::GpuRenderer;
use crate::text::TextSystem;
use crate::ui::{DrawContext, HitSink, Rect, Theme};
use crate::vault::Vault;
use crate::welcome::{WelcomeAction, WelcomeScreen};

/// Which screen the application is currently showing.
enum AppScreen {
    Welcome(WelcomeScreen),
    Editor(EditorView),
}

/// Top-level application state.
pub struct App<'window> {
    gpu: GpuRenderer<'window>,
    text_system: TextSystem,
    theme: Theme,
    hits: HitSink,
    screen: AppScreen,
    window: Option<Arc<Window>>,
    cursor_position: (f32, f32),
}

impl<'window> App<'window> {
    /// Creates the app in welcome-screen mode.
    pub fn new() -> Self {
        Self {
            gpu: GpuRenderer::new(),
            text_system: TextSystem::new(),
            theme: Theme::dark(),
            hits: HitSink::new(),
            screen: AppScreen::Welcome(WelcomeScreen::new()),
            window: None,
            cursor_position: (0.0, 0.0),
        }
    }

    fn handle_vault_action(&mut self, action: WelcomeAction) {
        match action {
            WelcomeAction::CreateVault => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Choose a folder for your new vault")
                    .pick_folder()
                {
                    match Vault::create(&path) {
                        Ok(vault) => {
                            let _ = register_vault(path);
                            self.screen = AppScreen::Editor(EditorView::new(&vault));
                        }
                        Err(error) => log::error!("Failed to create vault: {error}"),
                    }
                }
            }
            WelcomeAction::OpenVault => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Open a folder as vault")
                    .pick_folder()
                {
                    match Vault::open(&path) {
                        Ok(vault) => {
                            let _ = register_vault(path);
                            self.screen = AppScreen::Editor(EditorView::new(&vault));
                        }
                        Err(error) => log::error!("Failed to open vault: {error}"),
                    }
                }
            }
            WelcomeAction::None => {}
        }
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        match self.gpu.resume(event_loop) {
            Ok(window) => self.window = Some(window),
            Err(error) => {
                log::error!("Failed to create window: {error}");
                event_loop.exit();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.gpu.suspend();
        self.window = None;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.gpu.resize(size.width, size.height);
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                let Some(ref window) = self.window else {
                    return;
                };

                let scale = window.scale_factor() as f32;
                let physical = window.inner_size();
                let logical_width = physical.width as f32 / scale;
                let logical_height = physical.height as f32 / scale;
                let mut logical_scene = Scene::new();

                self.hits.clear();

                {
                    let mut ctx = DrawContext {
                        scene: &mut logical_scene,
                        text: &mut self.text_system,
                        theme: &self.theme,
                    };

                    let bounds = Rect::new(0.0, 0.0, logical_width, logical_height);

                    match &self.screen {
                        AppScreen::Welcome(welcome) => {
                            if let Err(error) = welcome.render(&mut ctx, &mut self.hits, bounds) {
                                log::error!("Welcome layout error: {error}");
                            }
                        }
                        AppScreen::Editor(editor) => {
                            if let Err(error) = editor.render(&mut ctx, &mut self.hits, bounds) {
                                log::error!("Editor layout error: {error}");
                            }
                        }
                    }
                }

                let mut scene = Scene::new();
                let scale_transform = vello::kurbo::Affine::scale(scale as f64);
                scene.append(&logical_scene, Some(scale_transform));

                if let Err(error) = self.gpu.render(&scene, self.theme.background) {
                    log::error!("Render error: {error}");
                }

                window.request_redraw();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let scale = self
                    .window
                    .as_ref()
                    .map(|w| w.scale_factor() as f32)
                    .unwrap_or(1.0);
                self.cursor_position = (position.x as f32 / scale, position.y as f32 / scale);
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(hit_id) = self
                    .hits
                    .test(self.cursor_position.0, self.cursor_position.1)
                {
                    match &mut self.screen {
                        AppScreen::Welcome(_) => {
                            self.handle_vault_action(WelcomeAction::from_hit(hit_id));
                        }
                        AppScreen::Editor(editor) => {
                            if EditorView::is_file_hit(hit_id) {
                                editor.handle_click(hit_id);
                            }
                        }
                    }
                }
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if let AppScreen::Welcome(_) = self.screen {
                    match logical_key {
                        Key::Character(ref ch) if ch.as_str() == "c" => {
                            self.handle_vault_action(WelcomeAction::CreateVault);
                        }
                        Key::Character(ref ch) if ch.as_str() == "o" => {
                            self.handle_vault_action(WelcomeAction::OpenVault);
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }
}
