use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key as WKey, ModifiersState, NamedKey},
    window::{Window, WindowId},
};

use crate::editor::Tab;
use crate::render::Renderer;
use crate::vim::Key;

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    tab: Tab,
    modifiers: ModifiersState,
}

impl App {
    /// Creates the app with a default buffer; renderer is initialised lazily on resume.
    pub fn new() -> Self {
        App {
            window: None,
            renderer: None,
            tab: Tab::new("# Hello, Onyx!\n\nStart typing...\n\n- Item one\n- Item two\n"),
            modifiers: ModifiersState::empty(),
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
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers = state.state();
            }
            WindowEvent::RedrawRequested => {
                self.tab.sync_document();
                if let Some(renderer) = &mut self.renderer {
                    renderer.scene.reset();
                    let render_lines = self.tab.editor.build_render_lines(
                        &self.tab.document,
                        self.tab.view_mode,
                    );
                    let cursor = self.tab.editor.buffer.cursor();
                    renderer.draw_render_lines(&render_lines, cursor.line, cursor.col);
                    renderer.render();
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }

                // ctrl+t toggles Live Preview / Raw mode for the active tab.
                if let WKey::Character(ref s) = event.logical_key {
                    if s == "t" && self.modifiers.control_key() {
                        self.tab.toggle_view_mode();
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                        return;
                    }
                }

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
                    self.tab.editor.handle_key(k);
                    self.tab.mark_dirty();
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }
}
