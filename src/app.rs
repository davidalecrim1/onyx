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
use crate::render::ui::{draw_file_tree, draw_tab_bar, FILE_TREE_WIDTH, TAB_HEIGHT};
use crate::render::Renderer;
use crate::shell::{
    CommandRegistry, EventBus, FileTree, GlobalConfig, KeyBindings, VaultConfig,
};
use crate::terminal::TerminalPane;
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
    commands: CommandRegistry,
    events: EventBus,
    keybindings: KeyBindings,
    file_tree: Option<FileTree>,
    file_tree_visible: bool,
    terminal_pane: Option<TerminalPane>,
    terminal_visible: bool,
    terminal_focused: bool,
    scale_factor: f32,
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

        let mut commands = CommandRegistry::new();
        commands.register("file.save", || {});
        commands.register("pane.file_tree.toggle", || {});
        commands.register("pane.terminal.toggle", || {});
        commands.register("pane.terminal.focus", || {});
        commands.register("terminal.new_tab", || {});
        commands.register("terminal.close_tab", || {});
        commands.register("command_palette.open", || {});

        App {
            window: None,
            renderer: None,
            tab,
            modifiers: ModifiersState::empty(),
            state,
            global_config,
            commands,
            events: EventBus::new(),
            keybindings: KeyBindings::load_for_platform(),
            file_tree: None,
            file_tree_visible: false,
            terminal_pane: None,
            terminal_visible: false,
            terminal_focused: false,
            scale_factor: 1.0,
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
        self.file_tree = Some(FileTree::new(&path));
        self.terminal_pane = Some(TerminalPane::new(&path, 24, 80));
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

    fn handle_named_command(&mut self, name: &str) {
        match name {
            "file.save" => self.save_vault_state(),
            "pane.file_tree.toggle" => {
                self.file_tree_visible = !self.file_tree_visible;
                self.events.emit("pane.toggled", "file_tree");
            }
            "pane.terminal.toggle" => {
                self.terminal_visible = !self.terminal_visible;
                self.events.emit("pane.toggled", "terminal");
            }
            "pane.terminal.focus" => {
                self.terminal_visible = true;
                self.terminal_focused = true;
            }
            "terminal.new_tab" => {
                if let Some(tp) = &mut self.terminal_pane {
                    tp.new_tab();
                }
            }
            "terminal.close_tab" => {
                if let Some(tp) = &mut self.terminal_pane {
                    tp.close_tab();
                }
            }
            "command_palette.open" => {
                eprintln!("[command palette] TODO");
            }
            _ => {
                self.commands.execute(name);
            }
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Returns the current system clipboard text, if accessible.
fn get_clipboard() -> Option<String> {
    arboard::Clipboard::new().ok()?.get_text().ok()
}

/// Converts a winit key event into the byte sequence the pty expects.
fn key_to_bytes(key: &WKey, modifiers: &ModifiersState) -> Vec<u8> {
    match key {
        WKey::Character(s) => {
            if modifiers.control_key() {
                if let Some(c) = s.chars().next() {
                    let lower = c.to_ascii_lowercase();
                    if lower >= 'a' && lower <= 'z' {
                        return vec![lower as u8 - b'a' + 1];
                    }
                }
            }
            s.as_bytes().to_vec()
        }
        WKey::Named(NamedKey::Enter)      => vec![b'\r'],
        WKey::Named(NamedKey::Backspace)  => vec![127],
        WKey::Named(NamedKey::Escape)     => vec![27],
        WKey::Named(NamedKey::ArrowUp)    => vec![27, b'[', b'A'],
        WKey::Named(NamedKey::ArrowDown)  => vec![27, b'[', b'B'],
        WKey::Named(NamedKey::ArrowRight) => vec![27, b'[', b'C'],
        WKey::Named(NamedKey::ArrowLeft)  => vec![27, b'[', b'D'],
        _ => vec![],
    }
}

/// Builds a chord string like "cmd+s" or "cmd+option+b" from a key event.
fn build_chord(logical_key: &WKey, modifiers: &ModifiersState) -> Option<String> {
    let mut parts = Vec::new();
    if modifiers.super_key()   { parts.push("cmd"); }
    if modifiers.alt_key()     { parts.push("option"); }
    if modifiers.control_key() { parts.push("ctrl"); }
    if modifiers.shift_key()   { parts.push("shift"); }
    if let WKey::Character(s) = logical_key {
        parts.push(s.as_str());
        return Some(parts.join("+"));
    }
    None
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Onyx"))
                .expect("failed to create window"),
        );
        self.scale_factor = window.scale_factor() as f32;
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
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor as f32;
            }
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers = state.state();
            }
            WindowEvent::RedrawRequested => {
                if let Some(tp) = &mut self.terminal_pane {
                    tp.tick_all();
                }

                if let Some(renderer) = &mut self.renderer {
                    renderer.scene.reset();

                    let surface_width = renderer.surface_width();
                    let surface_height = renderer.surface_height();

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
                            renderer.draw_render_lines(&lines, usize::MAX, 0, self.scale_factor);
                        }
                        AppState::Editor { .. } => {
                            draw_tab_bar(
                                &mut renderer.scene,
                                &[self.tab.file_path.as_ref()
                                    .and_then(|p| p.file_name())
                                    .map(|n| n.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| "untitled.md".into())],
                                0,
                                surface_width,
                            );

                            if self.file_tree_visible {
                                let entries = self.file_tree.as_ref()
                                    .map(|ft| ft.entries())
                                    .unwrap_or_default();
                                draw_file_tree(&mut renderer.scene, &entries, None, surface_height);
                            }

                            let editor_left = if self.file_tree_visible {
                                FILE_TREE_WIDTH
                            } else {
                                0.0
                            };
                            let _ = editor_left;

                            self.tab.sync_document();
                            let render_lines = self.tab.editor.build_render_lines(
                                &self.tab.document,
                                self.tab.view_mode,
                            );
                            let cursor = self.tab.editor.buffer.cursor();
                            renderer.draw_render_lines_offset(
                                &render_lines,
                                cursor.line,
                                cursor.col,
                                TAB_HEIGHT,
                                self.scale_factor,
                            );

                            if self.terminal_visible {
                                if let Some(tp) = &mut self.terminal_pane {
                                    let session = tp.active_session();
                                    let cell_width = 9.0_f32;
                                    let cell_height = 18.0_f32;
                                    let terminal_x = surface_width - 80.0 * cell_width;
                                    crate::render::terminal::draw_terminal(
                                        &mut renderer.scene,
                                        &session.performer.grid,
                                        terminal_x,
                                        TAB_HEIGHT,
                                        cell_width,
                                        cell_height,
                                    );
                                }
                            }
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

                if self.terminal_focused {
                    if let Some(tp) = &mut self.terminal_pane {
                        if self.modifiers.super_key() {
                            if let WKey::Character(s) = &event.logical_key {
                                match s.as_str() {
                                    "c" => { tp.active_session().write(&[3]); }
                                    "v" => {
                                        if let Some(text) = get_clipboard() {
                                            tp.active_session().write(text.as_bytes());
                                        }
                                    }
                                    _ => {}
                                }
                                if let Some(window) = &self.window { window.request_redraw(); }
                                return;
                            }
                        }
                        let bytes = key_to_bytes(&event.logical_key, &self.modifiers);
                        if !bytes.is_empty() {
                            tp.active_session().write(&bytes);
                        }
                    }
                    if let Some(window) = &self.window { window.request_redraw(); }
                    return;
                }

                if let Some(chord) = build_chord(&event.logical_key, &self.modifiers) {
                    if let Some(cmd_name) = self.keybindings.resolve(&chord) {
                        let cmd_name = cmd_name.to_string();
                        self.handle_named_command(&cmd_name);
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
