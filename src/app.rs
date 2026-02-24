use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key as WKey, NamedKey},
    window::{Window, WindowId},
};

use crate::editor::Editor;
use crate::render::Renderer;
use crate::vim::Key;

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    editor: Editor,
}

impl App {
    /// Creates the app with a default buffer; renderer is initialised lazily on resume.
    pub fn new() -> Self {
        App {
            window: None,
            renderer: None,
            editor: Editor::new("Hello, Onyx!\nStart typing..."),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Onyx"))
                .expect("failed to create window"),
        );
        let renderer = Renderer::new(window.clone());
        self.window = Some(window);
        self.renderer = Some(renderer);
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
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.scene.reset();
                    let lines: Vec<String> = (0..self.editor.buffer.line_count())
                        .map(|i| self.editor.buffer.line(i))
                        .collect();
                    let cursor = self.editor.buffer.cursor();
                    renderer.draw_buffer(&lines, cursor.line, cursor.col);
                    renderer.render();
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let key = match &event.logical_key {
                        WKey::Named(NamedKey::Escape)     => Some(Key::Escape),
                        WKey::Named(NamedKey::Backspace)  => Some(Key::Backspace),
                        WKey::Named(NamedKey::Enter)      => Some(Key::Enter),
                        WKey::Named(NamedKey::ArrowLeft)  => Some(Key::Left),
                        WKey::Named(NamedKey::ArrowRight) => Some(Key::Right),
                        WKey::Named(NamedKey::ArrowUp)    => Some(Key::Up),
                        WKey::Named(NamedKey::ArrowDown)  => Some(Key::Down),
                        WKey::Character(s) => s.chars().next().map(Key::Char),
                        _ => None,
                    };
                    if let Some(k) = key {
                        self.editor.handle_key(k);
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
