# Milestone 4: Vault System

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement vault open/create flow, per-vault `.onyx/config.toml`, global `~/.config/onyx/config.toml`, first-launch welcome screen, and state restoration on subsequent launches.

**Architecture:** A vault is a directory opened in Onyx. `src/shell/vault.rs` owns vault config I/O. On launch, `main.rs` reads the global config; if no vaults are known it renders a welcome screen; otherwise it reopens the last active vault. Each vault window owns its own `Tab` set and persists cursor positions and view modes to `.onyx/config.toml` on save and on close.

**Tech Stack:** `serde 1`, `toml 0.8`, standard `std::fs`, `rfd 0.14` (file picker dialog)

---

## Prerequisites

Milestone 3 complete: editor renders Markdown, tab holds view mode.

---

### Task 1: Add config types and serialisation

**Files:**
- Create: `src/shell/mod.rs`
- Create: `src/shell/vault.rs`
- Modify: `Cargo.toml`
- Modify: `src/main.rs`

**Step 1: Add dependencies to `Cargo.toml`**

```toml
serde = { version = "1", features = ["derive"] }
toml = "0.8"
rfd = "0.14"
```

**Step 2: Write failing tests**

```rust
// src/shell/vault.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn vault_config_round_trips() {
        let config = VaultConfig {
            open_tabs: vec![TabState {
                file_path: "notes.md".into(),
                cursor_line: 3,
                cursor_col: 7,
                view_mode: ViewModeState::LivePreview,
            }],
            pane_layout: PaneLayout::default(),
        };
        let toml = toml::to_string(&config).unwrap();
        let decoded: VaultConfig = toml::from_str(&toml).unwrap();
        assert_eq!(decoded.open_tabs[0].cursor_line, 3);
    }

    #[test]
    fn global_config_round_trips() {
        let config = GlobalConfig {
            vaults: vec![VaultEntry {
                name: "my-notes".into(),
                path: PathBuf::from("/Users/test/notes"),
            }],
            last_active: vec![PathBuf::from("/Users/test/notes")],
        };
        let toml = toml::to_string(&config).unwrap();
        let decoded: GlobalConfig = toml::from_str(&toml).unwrap();
        assert_eq!(decoded.vaults[0].name, "my-notes");
    }
}
```

**Step 3: Run to confirm failure**

```bash
cargo test shell 2>&1
```

Expected: compile error — types not found.

**Step 4: Implement `src/shell/vault.rs`**

```rust
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

// ── Per-vault config (.onyx/config.toml) ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewModeState {
    LivePreview,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub file_path: PathBuf,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub view_mode: ViewModeState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaneLayout {
    pub file_tree_position: Option<String>, // "left" | "right" | "bottom" | "top"
    pub terminal_position: Option<String>,
    pub file_tree_visible: bool,
    pub terminal_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    #[serde(default)]
    pub open_tabs: Vec<TabState>,
    #[serde(default)]
    pub pane_layout: PaneLayout,
}

impl Default for VaultConfig {
    fn default() -> Self {
        VaultConfig {
            open_tabs: Vec::new(),
            pane_layout: PaneLayout {
                file_tree_position: Some("left".into()),
                terminal_position: Some("right".into()),
                file_tree_visible: true,
                terminal_visible: false,
            },
        }
    }
}

impl VaultConfig {
    pub fn load(vault_root: &Path) -> Self {
        let path = vault_root.join(".onyx").join("config.toml");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, vault_root: &Path) -> std::io::Result<()> {
        let dir = vault_root.join(".onyx");
        std::fs::create_dir_all(&dir)?;
        let toml = toml::to_string_pretty(self).expect("serialise vault config");
        std::fs::write(dir.join("config.toml"), toml)
    }
}

// ── Global config (~/.config/onyx/config.toml) ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub vaults: Vec<VaultEntry>,
    #[serde(default)]
    pub last_active: Vec<PathBuf>,
}

impl GlobalConfig {
    fn config_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("onyx")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        let toml = toml::to_string_pretty(self).expect("serialise global config");
        std::fs::write(path, toml)
    }

    pub fn add_vault(&mut self, name: String, path: PathBuf) {
        if !self.vaults.iter().any(|v| v.path == path) {
            self.vaults.push(VaultEntry { name, path: path.clone() });
        }
        self.last_active.retain(|p| *p != path);
        self.last_active.insert(0, path);
    }
}
```

> **Note:** Add `dirs-next = "0.1"` to `Cargo.toml` for `dirs_next::config_dir()`.

**Step 5: Add `dirs-next` to `Cargo.toml`**

```toml
dirs-next = "0.1"
```

**Step 6: Create `src/shell/mod.rs`**

```rust
pub mod vault;
pub use vault::{GlobalConfig, PaneLayout, TabState, VaultConfig, VaultEntry, ViewModeState};
```

**Step 7: Declare module in `src/main.rs`**

```rust
mod shell;
```

**Step 8: Run tests**

```bash
cargo test shell 2>&1
```

Expected: both tests pass.

**Step 9: Stage changes**

```bash
git add .
```

**Step 10: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 2: First-launch welcome screen state

**Files:**
- Modify: `src/main.rs`

On first launch, the global config has no vaults. The app renders a welcome screen. This task adds the state machine that determines which screen to show.

**Step 1: Write the failing test**

```rust
// src/shell/vault.rs — tests block

#[test]
fn empty_global_config_means_first_launch() {
    let config = GlobalConfig::default();
    assert!(config.last_active.is_empty());
}
```

**Step 2: Run to confirm it passes immediately** (it should — default is empty)

```bash
cargo test shell 2>&1
```

Expected: passes.

**Step 3: Add `AppState` enum to `src/main.rs`**

```rust
use shell::{GlobalConfig, VaultConfig};
use std::path::PathBuf;

enum AppState {
    Welcome,
    Editor { vault_root: PathBuf, vault_config: VaultConfig },
}
```

**Step 4: Update `App` struct**

```rust
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    tab: Tab,
    state: AppState,
    global_config: GlobalConfig,
}
```

**Step 5: Initialise `state` in `main`**

```rust
let global_config = GlobalConfig::load();
let state = if global_config.last_active.is_empty() {
    AppState::Welcome
} else {
    let vault_root = global_config.last_active[0].clone();
    let vault_config = VaultConfig::load(&vault_root);
    AppState::Editor { vault_root, vault_config }
};

let mut app = App {
    window: None,
    renderer: None,
    tab: Tab::new(""),
    state,
    global_config,
};
```

**Step 6: In `RedrawRequested`, branch on `AppState`**

```rust
WindowEvent::RedrawRequested => {
    if let Some(r) = &mut self.renderer {
        r.scene.reset();
        match &self.state {
            AppState::Welcome => {
                // Render welcome screen placeholder text.
                let lines = vec![
                    crate::editor::RenderLine {
                        spans: vec![crate::editor::RenderSpan {
                            text: "Welcome to Onyx".into(),
                            style: crate::editor::SpanStyle::Heading(1),
                            is_raw: false,
                        }],
                    },
                    crate::editor::RenderLine {
                        spans: vec![crate::editor::RenderSpan {
                            text: "Press O to open a vault  •  C to create a vault".into(),
                            style: crate::editor::SpanStyle::Normal,
                            is_raw: false,
                        }],
                    },
                ];
                r.draw_render_lines(&lines, usize::MAX, 0);
            }
            AppState::Editor { .. } => {
                self.tab.sync_document();
                let render_lines = self.tab.editor.build_render_lines(
                    &self.tab.document,
                    self.tab.view_mode,
                    self.tab.editor.buffer.cursor(),
                );
                let cursor = self.tab.editor.buffer.cursor();
                r.draw_render_lines(&render_lines, cursor.line, cursor.col);
            }
        }
        r.render();
    }
    if let Some(w) = &self.window {
        w.request_redraw();
    }
}
```

**Step 7: Build and run**

```bash
cargo run
```

Expected: If no `~/.config/onyx/config.toml` exists, the Welcome screen text renders. Otherwise the last vault's editor opens.

**Step 8: Stage changes**

```bash
git add .
```

**Step 9: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 3: Open vault via file picker

**Files:**
- Modify: `src/main.rs`

When the welcome screen is visible, pressing `O` opens the native file picker (folder selection). On selection, the vault is registered and the editor opens.

**Step 1: Add folder picker helper to `src/main.rs`**

```rust
fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}
```

**Step 2: Handle `O` and `C` keypresses on the welcome screen**

In the keyboard handler, add a guard before routing to the Vim engine:

```rust
if let AppState::Welcome = &self.state {
    if let WKey::Character(s) = &logical_key {
        match s.as_str() {
            "o" | "O" => {
                if let Some(path) = pick_folder() {
                    self.open_vault(path);
                }
            }
            "c" | "C" => {
                if let Some(path) = pick_folder() {
                    std::fs::create_dir_all(&path).ok();
                    self.open_vault(path);
                }
            }
            _ => {}
        }
    }
    if let Some(w) = &self.window {
        w.request_redraw();
    }
    return;
}
```

**Step 3: Implement `open_vault` on `App`**

```rust
impl App {
    fn open_vault(&mut self, path: PathBuf) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "vault".into());

        self.global_config.add_vault(name, path.clone());
        self.global_config.save().ok();

        let vault_config = VaultConfig::load(&path);
        // Restore first tab if persisted.
        let initial_text = vault_config.open_tabs.first().and_then(|t| {
            std::fs::read_to_string(path.join(&t.file_path)).ok()
        }).unwrap_or_default();

        self.tab = Tab::new(&initial_text);
        self.state = AppState::Editor { vault_root: path, vault_config };

        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}
```

**Step 4: Build and run**

```bash
cargo run
```

Expected: Welcome screen appears. Press `O`, system folder picker opens, select any folder. Editor opens with an empty buffer. `~/.config/onyx/config.toml` is created.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 4: Persist vault state on save and close

**Files:**
- Modify: `src/main.rs`

When `cmd+s` is pressed or the window closes, write the current tab state to `.onyx/config.toml`.

**Step 1: Add `save_vault_state` to `App`**

```rust
impl App {
    fn save_vault_state(&self) {
        let AppState::Editor { vault_root, vault_config: _ } = &self.state else { return };

        let cursor = self.tab.editor.buffer.cursor();
        let view_mode = match self.tab.view_mode {
            crate::editor::ViewMode::LivePreview => shell::ViewModeState::LivePreview,
            crate::editor::ViewMode::Raw => shell::ViewModeState::Raw,
        };

        let tab_state = shell::TabState {
            file_path: self.tab.file_path.clone().unwrap_or_else(|| "untitled.md".into()),
            cursor_line: cursor.line,
            cursor_col: cursor.col,
            view_mode,
        };

        let config = shell::VaultConfig {
            open_tabs: vec![tab_state],
            ..shell::VaultConfig::default()
        };

        config.save(vault_root).ok();

        // Also write the file content if the tab has a path.
        if let Some(ref file_path) = self.tab.file_path {
            std::fs::write(vault_root.join(file_path), self.tab.editor.buffer_text()).ok();
        }
    }
}
```

**Step 2: Add `file_path` to `Tab`**

```rust
pub struct Tab {
    // ...existing fields...
    pub file_path: Option<std::path::PathBuf>,
}
```

**Step 3: Call `save_vault_state` on `cmd+s`**

In the keyboard handler, intercept `cmd+s` before Vim routing:

```rust
if let WKey::Character(s) = &logical_key {
    if s == "s" && modifiers.super_key() {
        self.save_vault_state();
        return;
    }
}
```

**Step 4: Call `save_vault_state` on window close**

In `WindowEvent::CloseRequested`:

```rust
WindowEvent::CloseRequested => {
    self.save_vault_state();
    event_loop.exit();
}
```

**Step 5: Build and run**

```bash
cargo run
```

Expected: After opening a vault, pressing `cmd+s` writes `.onyx/config.toml`. Close the window and reopen — the same vault loads (restoring from global config).

**Step 6: Stage changes**

```bash
git add .
```

**Step 7: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 5: Restore vault state on subsequent launch

**Files:**
- Modify: `src/main.rs`

The `AppState::Editor` branch in `main` already loads `vault_config`. This task restores the cursor position from the persisted tab state.

**Step 1: Restore cursor in `open_vault` (and on launch)**

Update the `AppState::Editor` initialisation in `main` to restore the cursor:

```rust
let state = if global_config.last_active.is_empty() {
    AppState::Welcome
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
    // Restore cursor by replaying moves (simplest correct approach for MVP).
    for _ in 0..cursor_line {
        tab.editor.buffer.move_down();
    }
    for _ in 0..cursor_col {
        tab.editor.buffer.move_right();
    }

    AppState::Editor { vault_root, vault_config }
};
```

> **Note:** `tab` needs to be stored in `App`. Refactor so `App` is built after the state is determined.

**Step 2: Build and run**

```bash
cargo run
```

Expected: Open a vault, navigate to line 5 column 3, press `cmd+s`, close, reopen. Cursor should restore to line 5 column 3.

**Step 3: Stage changes**

```bash
git add .
```

**Step 4: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

## Milestone 4 Complete

At this point:
- Global config at `~/.config/onyx/config.toml` tracks known vaults and last active
- Per-vault config at `.onyx/config.toml` tracks open tabs, cursor positions, and pane layout
- First launch shows a Welcome screen with Open Vault / Create Vault actions
- Subsequent launches reopen the last active vault and restore cursor positions
- `cmd+s` persists the current state; window close also persists
- iCloud: no code required — users point vault at `~/Library/Mobile Documents/`
