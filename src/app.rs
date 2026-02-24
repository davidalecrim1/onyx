use std::path::PathBuf;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key as WKey, ModifiersState, NamedKey},
    window::{Window, WindowId},
};

use crate::editor::{RenderLine, RenderSpan, SpanStyle, Tab};
use crate::render::Renderer;
use crate::shell::{GlobalConfig, VaultConfig};
use crate::vim::Key;

enum AppState {
    Welcome,
    Editor { vault_root: PathBuf, vault_config: VaultConfig },
}

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    tab: Tab,
    modifiers: ModifiersState,
    state: AppState,
    global_config: GlobalConfig,
}

impl App {
    /// Creates the app, loading global config and determining whether to show the welcome screen.
    pub fn new() -> Self {
        let global_config = GlobalConfig::load();
        let (state, tab) = if global_config.last_active.is_empty() {
            (AppState::Welcome, Tab::new(""))
        } else {
            let vault_root = global_config.last_active[0].clone();
            let vault_config = VaultConfig::load(&vault_root);
            let (initial_text, cursor_line, cursor_col) = vault_config
                .open_tabs
                .first()
                .and_then(|t| {
                    let text = std::fs::read_to_string(vault_root.join(&t.file_path)).ok()?;
                    Some((text, t.cursor_line, t.cursor_col))
                })
                .unwrap_or_default();
            let mut tab = Tab::new(&initial_text);
            for _ in 0..cursor_line {
                tab.editor.buffer.move_down();
            }
            for _ in 0..cursor_col {
                tab.editor.buffer.move_right();
            }
            (AppState::Editor { vault_root, vault_config }, tab)
        };

        App {
            window: None,
            renderer: None,
            tab,
            modifiers: ModifiersState::empty(),
            state,
            global_config,
        }
    }

    fn open_vault(&mut self, path: PathBuf) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "vault".into());

        self.global_config.add_vault(name, path.clone());
        self.global_config.save().ok();

        let vault_config = VaultConfig::load(&path);
        let initial_text = vault_config.open_tabs.first()
            .and_then(|t| std::fs::read_to_string(path.join(&t.file_path)).ok())
            .unwrap_or_default();

        self.tab = Tab::new(&initial_text);
        self.state = AppState::Editor { vault_root: path, vault_config };

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn save_vault_state(&self) {
        let AppState::Editor { vault_root, .. } = &self.state else { return };

        let cursor = self.tab.editor.buffer.cursor();
        let view_mode = match self.tab.view_mode {
            crate::editor::ViewMode::LivePreview => crate::shell::ViewModeState::LivePreview,
            crate::editor::ViewMode::Raw => crate::shell::ViewModeState::Raw,
        };

        let tab_state = crate::shell::TabState {
            file_path: self.tab.file_path.clone().unwrap_or_else(|| "untitled.md".into()),
            cursor_line: cursor.line,
            cursor_col: cursor.col,
            view_mode,
        };

        let config = crate::shell::VaultConfig {
            open_tabs: vec![tab_state],
            ..crate::shell::VaultConfig::default()
        };

        config.save(vault_root).ok();

        if let Some(ref file_path) = self.tab.file_path {
            std::fs::write(vault_root.join(file_path), self.tab.editor.buffer_text()).ok();
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
            WindowEvent::CloseRequested => {
                self.save_vault_state();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size);
                }
            }
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers = state.state();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.scene.reset();
                    match &self.state {
                        AppState::Welcome => {
                            let lines = vec![
                                RenderLine {
                                    spans: vec![RenderSpan {
                                        text: "Welcome to Onyx".into(),
                                        style: SpanStyle::Heading(1),
                                        is_raw: false,
                                    }],
                                },
                                RenderLine {
                                    spans: vec![RenderSpan {
                                        text: "Press O to open a vault  •  C to create a vault".into(),
                                        style: SpanStyle::Normal,
                                        is_raw: false,
                                    }],
                                },
                            ];
                            renderer.draw_render_lines(&lines, usize::MAX, 0);
                        }
                        AppState::Editor { .. } => {
                            self.tab.sync_document();
                            let render_lines = self.tab.editor.build_render_lines(
                                &self.tab.document,
                                self.tab.view_mode,
                            );
                            let cursor = self.tab.editor.buffer.cursor();
                            renderer.draw_render_lines(&render_lines, cursor.line, cursor.col);
                        }
                    }
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

                // Welcome screen handles only O and C.
                if let AppState::Welcome = &self.state {
                    if let WKey::Character(s) = &event.logical_key {
                        match s.as_str() {
                            "o" | "O" => {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    self.open_vault(path);
                                }
                            }
                            "c" | "C" => {
                                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                    std::fs::create_dir_all(&path).ok();
                                    self.open_vault(path);
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                    return;
                }

                // cmd+s saves vault state.
                if let WKey::Character(s) = &event.logical_key {
                    if s == "s" && self.modifiers.super_key() {
                        self.save_vault_state();
                        return;
                    }
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
