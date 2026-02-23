# Milestone 5: Workspace Shell

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the full workspace UI: file tree with file operations, tab bar with Live Preview / Raw toggle, split pane layout, command palette, keybinding system, event bus, and command registry.

**Architecture:** `src/shell/` grows several sub-modules: `event_bus.rs` (Pub/Sub for named events), `command_registry.rs` (named commands → closures), `file_tree.rs` (recursive `.md` file listing with CRUD ops), `keybindings.rs` (loads `src/keybindings/macos.json` and resolves key chords to command names). `src/render/` grows `ui.rs` for drawing the tab bar, file tree, and pane dividers using Vello filled rectangles and text.

**Tech Stack:** `serde_json 1` (keybindings JSON), `notify 6` (optional file-watch), standard `std::fs`

---

## Prerequisites

Milestone 4 complete: vault opens, global + vault config persisted.

---

### Task 1: Command registry

**Files:**
- Create: `src/shell/command_registry.rs`
- Modify: `src/shell/mod.rs`

All actions in the app are registered as named commands. Keybindings and the command palette invoke commands by name — never by direct function call.

**Step 1: Write failing tests**

```rust
// src/shell/command_registry.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_command_is_callable() {
        let mut reg = CommandRegistry::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called2 = called.clone();
        reg.register("test.command", move || {
            called2.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        reg.execute("test.command");
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn unknown_command_does_not_panic() {
        let mut reg = CommandRegistry::new();
        reg.execute("does.not.exist"); // must not panic
    }

    #[test]
    fn list_commands_returns_all_registered() {
        let mut reg = CommandRegistry::new();
        reg.register("a.command", || {});
        reg.register("b.command", || {});
        let names = reg.command_names();
        assert!(names.contains(&"a.command"));
        assert!(names.contains(&"b.command"));
    }
}
```

**Step 2: Run to confirm failure**

```bash
cargo test command_registry 2>&1
```

Expected: compile error — `CommandRegistry` not found.

**Step 3: Implement `src/shell/command_registry.rs`**

```rust
use std::collections::HashMap;

type CommandFn = Box<dyn FnMut() + Send>;

pub struct CommandRegistry {
    commands: HashMap<String, CommandFn>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        CommandRegistry { commands: HashMap::new() }
    }

    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: FnMut() + Send + 'static,
    {
        self.commands.insert(name.to_string(), Box::new(f));
    }

    pub fn execute(&mut self, name: &str) {
        if let Some(cmd) = self.commands.get_mut(name) {
            cmd();
        }
    }

    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }
}
```

**Step 4: Run tests**

```bash
cargo test command_registry 2>&1
```

Expected: all 3 tests pass.

**Step 5: Export from `src/shell/mod.rs`**

```rust
pub mod command_registry;
pub use command_registry::CommandRegistry;
```

**Step 6: Stage changes**

```bash
git add .
```

**Step 7: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 2: Event bus

**Files:**
- Create: `src/shell/event_bus.rs`
- Modify: `src/shell/mod.rs`

The event bus allows internal components (and post-MVP plugins) to subscribe to named events.

**Step 1: Write failing tests**

```rust
// src/shell/event_bus.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscriber_receives_event() {
        let mut bus = EventBus::new();
        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count2 = count.clone();
        bus.subscribe("buffer.changed", move |_payload| {
            count2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        bus.emit("buffer.changed", "");
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn multiple_subscribers_all_called() {
        let mut bus = EventBus::new();
        let a = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let b = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let a2 = a.clone();
        let b2 = b.clone();
        bus.subscribe("file.opened", move |_| { a2.store(true, std::sync::atomic::Ordering::SeqCst); });
        bus.subscribe("file.opened", move |_| { b2.store(true, std::sync::atomic::Ordering::SeqCst); });
        bus.emit("file.opened", "/path/to/file.md");
        assert!(a.load(std::sync::atomic::Ordering::SeqCst));
        assert!(b.load(std::sync::atomic::Ordering::SeqCst));
    }
}
```

**Step 2: Run to confirm failure**

```bash
cargo test event_bus 2>&1
```

Expected: compile error.

**Step 3: Implement `src/shell/event_bus.rs`**

```rust
use std::collections::HashMap;

type HandlerFn = Box<dyn FnMut(&str) + Send>;

pub struct EventBus {
    handlers: HashMap<String, Vec<HandlerFn>>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus { handlers: HashMap::new() }
    }

    pub fn subscribe<F>(&mut self, event: &str, f: F)
    where
        F: FnMut(&str) + Send + 'static,
    {
        self.handlers.entry(event.to_string()).or_default().push(Box::new(f));
    }

    pub fn emit(&mut self, event: &str, payload: &str) {
        if let Some(handlers) = self.handlers.get_mut(event) {
            for h in handlers.iter_mut() {
                h(payload);
            }
        }
    }
}
```

**Step 4: Run tests**

```bash
cargo test event_bus 2>&1
```

Expected: both tests pass.

**Step 5: Export from `src/shell/mod.rs`**

```rust
pub mod event_bus;
pub use event_bus::EventBus;
```

**Step 6: Stage changes**

```bash
git add .
```

**Step 7: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 3: Keybinding system

**Files:**
- Create: `src/keybindings/macos.json`
- Create: `src/shell/keybindings.rs`
- Modify: `src/shell/mod.rs`
- Modify: `Cargo.toml`

Keybindings are loaded from a JSON file at startup. A `KeyBindings` struct resolves a key chord to a command name.

**Step 1: Add `serde_json` to `Cargo.toml`**

```toml
serde_json = "1"
```

**Step 2: Write the macOS keybindings file**

Create `src/keybindings/macos.json`:

```json
{
  "cmd+b":         "pane.file_tree.toggle",
  "cmd+option+b":  "pane.terminal.toggle",
  "cmd+j":         "pane.terminal.focus",
  "cmd+s":         "file.save",
  "cmd+p":         "command_palette.open",
  "cmd+t":         "terminal.new_tab",
  "cmd+w":         "terminal.close_tab"
}
```

**Step 3: Write failing test**

```rust
// src/shell/keybindings.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_s_resolves_to_file_save() {
        let kb = KeyBindings::from_json(r#"{"cmd+s": "file.save"}"#);
        assert_eq!(kb.resolve("cmd+s"), Some("file.save"));
    }

    #[test]
    fn unknown_chord_returns_none() {
        let kb = KeyBindings::from_json(r#"{}"#);
        assert_eq!(kb.resolve("cmd+z"), None);
    }
}
```

**Step 4: Run to confirm failure**

```bash
cargo test keybindings 2>&1
```

Expected: compile error.

**Step 5: Implement `src/shell/keybindings.rs`**

```rust
use std::collections::HashMap;

pub struct KeyBindings {
    map: HashMap<String, String>,
}

impl KeyBindings {
    pub fn from_json(json: &str) -> Self {
        let map: HashMap<String, String> =
            serde_json::from_str(json).unwrap_or_default();
        KeyBindings { map }
    }

    pub fn load_for_platform() -> Self {
        // Detect platform and load the correct file.
        #[cfg(target_os = "macos")]
        let json = include_str!("../keybindings/macos.json");
        #[cfg(not(target_os = "macos"))]
        let json = "{}"; // stubs for non-macOS platforms

        Self::from_json(json)
    }

    /// Returns the command name for a chord, or None if not bound.
    pub fn resolve(&self, chord: &str) -> Option<&str> {
        self.map.get(chord).map(|s| s.as_str())
    }
}
```

**Step 6: Run tests**

```bash
cargo test keybindings 2>&1
```

Expected: both tests pass.

**Step 7: Export from `src/shell/mod.rs`**

```rust
pub mod keybindings;
pub use keybindings::KeyBindings;
```

**Step 8: Stage changes**

```bash
git add .
```

**Step 9: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 4: File tree model

**Files:**
- Create: `src/shell/file_tree.rs`
- Modify: `src/shell/mod.rs`

The file tree scans the vault root for `.md` files recursively and supports create, rename, delete, and move operations.

**Step 1: Write failing tests**

```rust
// src/shell/file_tree.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_entry(name: &str) -> FileEntry {
        FileEntry { path: PathBuf::from(name), is_dir: false }
    }

    #[test]
    fn new_file_creation() {
        let dir = tempfile::tempdir().unwrap();
        let tree = FileTree::new(dir.path());
        tree.create_file("notes.md").unwrap();
        assert!(dir.path().join("notes.md").exists());
    }

    #[test]
    fn delete_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("old.md"), "").unwrap();
        let tree = FileTree::new(dir.path());
        tree.delete_file("old.md").unwrap();
        assert!(!dir.path().join("old.md").exists());
    }

    #[test]
    fn rename_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.md"), "").unwrap();
        let tree = FileTree::new(dir.path());
        tree.rename_file("a.md", "b.md").unwrap();
        assert!(dir.path().join("b.md").exists());
        assert!(!dir.path().join("a.md").exists());
    }
}
```

**Step 2: Add `tempfile` to `Cargo.toml`** (dev-dependency only)

```toml
[dev-dependencies]
tempfile = "3"
```

**Step 3: Run to confirm failure**

```bash
cargo test file_tree 2>&1
```

Expected: compile error.

**Step 4: Implement `src/shell/file_tree.rs`**

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct FileTree {
    root: PathBuf,
}

impl FileTree {
    pub fn new(root: &Path) -> Self {
        FileTree { root: root.to_path_buf() }
    }

    /// Returns all .md files in the vault root, sorted by path.
    pub fn entries(&self) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        self.collect_entries(&self.root, &mut entries);
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        entries
    }

    fn collect_entries(&self, dir: &Path, out: &mut Vec<FileEntry>) {
        let Ok(read_dir) = std::fs::read_dir(dir) else { return };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                if name.starts_with('.') { continue; } // skip .onyx, .git, etc.
                out.push(FileEntry { path: self.relative(&path), is_dir: true });
                self.collect_entries(&path, out);
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                out.push(FileEntry { path: self.relative(&path), is_dir: false });
            }
        }
    }

    fn relative(&self, path: &Path) -> PathBuf {
        path.strip_prefix(&self.root).unwrap_or(path).to_path_buf()
    }

    pub fn create_file(&self, name: &str) -> std::io::Result<()> {
        std::fs::write(self.root.join(name), "")
    }

    pub fn delete_file(&self, name: &str) -> std::io::Result<()> {
        std::fs::remove_file(self.root.join(name))
    }

    pub fn rename_file(&self, from: &str, to: &str) -> std::io::Result<()> {
        std::fs::rename(self.root.join(from), self.root.join(to))
    }

    pub fn move_file(&self, from: &str, to: &str) -> std::io::Result<()> {
        std::fs::rename(self.root.join(from), self.root.join(to))
    }
}
```

**Step 5: Run tests**

```bash
cargo test file_tree 2>&1
```

Expected: all 3 tests pass.

**Step 6: Export from `src/shell/mod.rs`**

```rust
pub mod file_tree;
pub use file_tree::{FileEntry, FileTree};
```

**Step 7: Stage changes**

```bash
git add .
```

**Step 8: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 5: Wire command registry into the event loop

**Files:**
- Modify: `src/main.rs`

Replace the ad-hoc `cmd+s`/`cmd+b` intercepts with proper command registry dispatch. Keybindings resolve chords to command names; the registry executes them.

**Step 1: Update `App` to hold registry, event bus, keybindings, and file tree**

```rust
use shell::{CommandRegistry, EventBus, FileTree, KeyBindings};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    tab: Tab,
    state: AppState,
    global_config: GlobalConfig,
    commands: CommandRegistry,
    events: EventBus,
    keybindings: KeyBindings,
    file_tree: Option<FileTree>,
    file_tree_visible: bool,
}
```

**Step 2: Register all commands in `main`**

```rust
let mut commands = CommandRegistry::new();
// Commands will close over app state via Arc<Mutex<>> in post-MVP.
// For MVP, register stubs that set atomic flags read in the event loop.
commands.register("file.save", || { /* handled directly for MVP */ });
commands.register("pane.file_tree.toggle", || { /* handled below */ });
commands.register("pane.terminal.toggle", || { /* Milestone 6 */ });
commands.register("pane.terminal.focus", || { /* Milestone 6 */ });
commands.register("command_palette.open", || { /* handled below */ });
```

**Step 3: Resolve keybindings in the keyboard handler**

```rust
// Build chord string from modifiers + key.
fn build_chord(logical_key: &WKey, modifiers: &winit::event::Modifiers) -> Option<String> {
    let mut parts = Vec::new();
    if modifiers.super_key()  { parts.push("cmd"); }
    if modifiers.alt_key()    { parts.push("option"); }
    if modifiers.control_key(){ parts.push("ctrl"); }
    if modifiers.shift_key()  { parts.push("shift"); }
    if let WKey::Character(s) = logical_key {
        parts.push(s.as_str());
        return Some(parts.join("+"));
    }
    None
}
```

In `KeyboardInput`:

```rust
if let Some(chord) = build_chord(&logical_key, &modifiers) {
    if let Some(cmd_name) = self.keybindings.resolve(&chord) {
        let cmd_name = cmd_name.to_string();
        self.handle_named_command(&cmd_name);
        return;
    }
}
```

**Step 4: Implement `handle_named_command`**

```rust
impl App {
    fn handle_named_command(&mut self, name: &str) {
        match name {
            "file.save" => self.save_vault_state(),
            "pane.file_tree.toggle" => {
                self.file_tree_visible = !self.file_tree_visible;
                self.events.emit("pane.toggled", "file_tree");
            }
            "command_palette.open" => {
                // Milestone 5 stretch: command palette overlay.
                // For now, log to stderr.
                eprintln!("[command palette] TODO");
            }
            _ => {
                self.commands.execute(name);
            }
        }
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}
```

**Step 5: Build and run**

```bash
cargo run
```

Expected: `cmd+s` triggers save via registry. `cmd+b` toggles a state flag (file tree not yet rendered — that is the next task).

**Step 6: Stage changes**

```bash
git add .
```

**Step 7: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 6: Render the workspace layout

**Files:**
- Create: `src/render/ui.rs`
- Modify: `src/render/mod.rs`
- Modify: `src/main.rs`

Draw the tab bar, file tree panel, and pane dividers using Vello filled rectangles and text.

**Step 1: Create `src/render/ui.rs`**

```rust
use crate::shell::FileEntry;
use vello::kurbo::{Affine, Line, Rect};
use vello::peniko::{Brush, Color, Fill, Stroke};
use vello::Scene;

pub const TAB_HEIGHT: f32 = 32.0;
pub const FILE_TREE_WIDTH: f32 = 220.0;
pub const DIVIDER_COLOR: Color = Color::from_rgba8(50, 50, 58, 255);
pub const TAB_BG: Color = Color::from_rgba8(30, 30, 36, 255);
pub const TAB_ACTIVE_BG: Color = Color::from_rgba8(40, 40, 48, 255);
pub const FILE_TREE_BG: Color = Color::from_rgba8(24, 24, 30, 255);

pub fn draw_tab_bar(
    scene: &mut Scene,
    tabs: &[String],
    active: usize,
    width: f32,
) {
    // Tab bar background.
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(TAB_BG),
        None,
        &Rect::new(0.0, 0.0, width as f64, TAB_HEIGHT as f64),
    );

    // Individual tab slots.
    let tab_width = 120.0_f32;
    for (i, name) in tabs.iter().enumerate() {
        let x = i as f32 * tab_width;
        let bg = if i == active { TAB_ACTIVE_BG } else { TAB_BG };
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(bg),
            None,
            &Rect::new(x as f64, 0.0, (x + tab_width) as f64, TAB_HEIGHT as f64),
        );
        // Tab name drawn by renderer text system (TODO: cosmic-text glyphs).
        let _ = name;
    }

    // Bottom border.
    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(DIVIDER_COLOR),
        None,
        &Line::new((0.0, TAB_HEIGHT as f64), (width as f64, TAB_HEIGHT as f64)),
    );
}

pub fn draw_file_tree(
    scene: &mut Scene,
    entries: &[FileEntry],
    selected: Option<usize>,
    height: f32,
) {
    // Panel background.
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(FILE_TREE_BG),
        None,
        &Rect::new(0.0, TAB_HEIGHT as f64, FILE_TREE_WIDTH as f64, height as f64),
    );

    // Right border.
    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        &Brush::Solid(DIVIDER_COLOR),
        None,
        &Line::new(
            (FILE_TREE_WIDTH as f64, TAB_HEIGHT as f64),
            (FILE_TREE_WIDTH as f64, height as f64),
        ),
    );

    // Entry rows (text drawn by renderer text system).
    let row_height = 22.0_f32;
    for (i, entry) in entries.iter().enumerate() {
        let y = TAB_HEIGHT + i as f32 * row_height;
        if selected == Some(i) {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::from_rgba8(50, 100, 180, 80)),
                None,
                &Rect::new(0.0, y as f64, FILE_TREE_WIDTH as f64, (y + row_height) as f64),
            );
        }
        let _ = entry;
    }
}
```

**Step 2: Re-export from `src/render/mod.rs`**

```rust
pub mod ui;
```

**Step 3: Call `draw_tab_bar` and `draw_file_tree` from `App::RedrawRequested`**

```rust
// Before drawing render lines:
use render::ui::{draw_file_tree, draw_tab_bar, FILE_TREE_WIDTH, TAB_HEIGHT};

if let Some(r) = &mut self.renderer {
    r.scene.reset();

    let w = r.config.width as f32;
    let h = r.config.height as f32;

    draw_tab_bar(&mut r.scene, &["untitled.md".to_string()], 0, w);

    let editor_x = if self.file_tree_visible { FILE_TREE_WIDTH } else { 0.0 };

    if self.file_tree_visible {
        let entries = self.file_tree.as_ref()
            .map(|ft| ft.entries())
            .unwrap_or_default();
        draw_file_tree(&mut r.scene, &entries, None, h);
    }

    // Offset render lines into the editor pane area.
    // (For MVP, draw_render_lines uses absolute coords — pass offset in future refactor.)
    // ...
}
```

**Step 4: Build and run**

```bash
cargo run
```

Expected: Tab bar visible at top of window. `cmd+b` toggles the file tree panel. Dark panel background and divider lines are visible.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

## Milestone 5 Complete

At this point:
- `src/shell/command_registry.rs`: all actions registered as named commands
- `src/shell/event_bus.rs`: pub/sub named events
- `src/shell/keybindings.rs`: JSON chord → command name resolution, macOS bindings loaded
- `src/shell/file_tree.rs`: `.md` file listing, create/rename/delete/move
- `src/render/ui.rs`: tab bar and file tree panel drawn via Vello
- Keybindings (`cmd+b`, `cmd+s`, `cmd+p`, `cmd+j`) all route through the command registry
- Per-vault pane layout persisted in `.onyx/config.toml`

**Command palette:** The `cmd+p` stub opens a `TODO` log. Full fuzzy search overlay over files and commands is a post-Milestone-5 stretch task — it requires a text input widget in the UI layer.
