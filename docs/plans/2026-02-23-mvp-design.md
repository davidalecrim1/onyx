# Onyx MVP Design

## Overview

Onyx is a native, GPU-rendered Markdown editor built entirely in Rust. Inspired by Zed's minimalist and performance-first philosophy, it targets developers who want an Obsidian-like experience with Vim-first editing, mouseless navigation, and a pluggable architecture.

**Target platform:** macOS only for MVP.
**Non-goals for MVP:** encryption, plugin loader, themes, mobile, Git sync.

---

## Section 1: Project Structure

Single Rust workspace using `cargo init .` convention. Each crate has a focused responsibility.

```
onyx/
├── src/
│   ├── buffer/      # Rope-based text buffer, cursor, selection
│   ├── vim/         # Vim modal engine (normal/insert/visual modes)
│   ├── markdown/    # Markdown parser → AST, diff engine
│   ├── render/      # wgpu + Vello + cosmic-text rendering pipeline
│   ├── editor/      # WYSIWYG layer: AST → render commands
│   ├── shell/       # Window, panels, file tree, tabs, command palette, event bus, command registry
│   ├── terminal/    # Embedded terminal pane (pty integration)
│   ├── keybindings/ # OS keybinding files (macos.json, linux.json, windows.json)
│   └── main.rs      # Entry point: winit event loop, wires everything together
├── docs/
└── Cargo.toml
```

**Data flow:**

```
User input → winit event loop (main.rs)
  → Command registry (shell) → resolves named command
  → Event bus (shell) → dispatches to subscribers
  → Vim engine (vim) → buffer mutation
  → Markdown parser (markdown) → AST diff
  → WYSIWYG renderer (editor) → render commands
  → GPU pipeline (render) → wgpu/Vello → screen
```

---

## Section 2: Technology Stack

| Layer | Crate |
|-------|-------|
| Windowing | `winit` |
| GPU | `wgpu` |
| Rendering | `vello` |
| Text layout | `cosmic-text` |
| Layout | `taffy` |
| Text buffer | `ropey` |
| Markdown parsing | `pulldown-cmark` |
| Terminal (pty) | `portable-pty` |
| Config serialization | `serde` + `toml` |

---

## Section 3: Vault System

A vault is a directory opened in Onyx. Each vault is independent — it opens in its own window with its own file tree, tabs, and persisted state.

**Vault config:** `.onyx/config.toml` inside the vault root stores:
- Open tabs and their view modes (live preview vs raw)
- Cursor positions per file
- Pane layout (file tree position, terminal position)

**Global config:** `~/.config/onyx/config.toml` stores:
- List of known vaults (name + path)
- Last active vault(s)

### First Launch

1. Welcome screen appears — no file tree, no tabs
2. Two actions: **Open Vault** (existing folder) or **Create Vault** (new folder)
3. On selection, Onyx creates `.onyx/config.toml` and opens the main editor window
4. Global config is written with the vault entry

### Subsequent Launches

1. Onyx reads `~/.config/onyx/config.toml`
2. Reopens last active vault(s) — restores window position, open tabs, cursor positions
3. If a vault directory is missing (e.g. iCloud not synced), shows a non-blocking warning with options to locate or remove it

### iCloud Sync

No code required. Users point their vault at a directory inside `~/Library/Mobile Documents/` and the OS handles sync transparently.

---

## Section 4: Editor Core

### Buffer (`src/buffer/`)

- Rope data structure via `ropey` for efficient insert/delete on large files
- One buffer per open file, owned by the vault session
- Cursor and selection state lives on the buffer

### Vim Engine (`src/vim/`)

- Modes: Normal, Insert, Visual (no Ex mode for MVP)
- Save via `cmd+s` (no `:w`)
- Keybindings mirror Zed's Vim mode

**In-scope motions and operators:**

| Category | Keys |
|----------|------|
| Movement | `h/j/k/l`, `w/b/e`, `0/$`, `gg/G`, `{/}` |
| Operators | `d`, `c`, `y`, `p` |
| Undo/redo | `u`, `ctrl+r` |
| Visual | character and line selection |

**Out of scope for MVP:** macros, marks, Ex commands (`:s`, `:g`), multiple cursors.

---

## Section 5: WYSIWYG Rendering

### Markdown Parser (`src/markdown/`)

- `pulldown-cmark` for CommonMark-compliant parsing
- On each edit, re-parse only the affected block — not the full document
- AST diff produces minimal render updates

### Editor Layer (`src/editor/`)

Live preview mode — Markdown syntax is hidden and replaced with styled output. Raw syntax is revealed when the cursor enters a construct.

| Element | Rendered as |
|---------|-------------|
| `# Heading` | Large styled text, `#` hidden |
| `**bold**` | Bold text, `**` hidden |
| `_italic_` | Italic text, `_` hidden |
| `` `code` `` | Monospace highlighted span |
| `code block` | Full block with background fill |
| `[link](url)` | Styled link text, URL hidden |
| `- item` | Bullet point, `-` hidden |

### View Modes

Each tab independently tracks its view mode, persisted in `.onyx/config.toml`:

- **Live preview** — WYSIWYG, syntax hidden, cursor reveals raw on entry
- **Raw mode** — plain Markdown source, syntax highlighting only

Toggle button lives in the tab bar. No default keybinding for MVP.

---

## Section 6: Workspace Shell

### Layout

Default layout:

```
┌─────────────────────────────────────────────────────┐
│  [Tab 1] [Tab 2] [Tab 3]            [raw|preview]   │  ← tab bar
├───────────┬─────────────────────────┬───────────────┤
│           │                         │               │
│ File tree │      Editor pane        │   Terminal    │
│  (left)   │                         │   (right)     │
│           │                         │               │
└───────────┴─────────────────────────┴───────────────┘
```

Both the file tree and terminal pane are movable to left/right/bottom/top via the command palette. Position is persisted per-vault.

### Default Keybindings

| Action | Keybinding |
|--------|------------|
| Toggle file tree (left pane) | `cmd+b` |
| Toggle terminal (right pane) | `cmd+option+b` |
| Open/focus terminal | `cmd+j` |
| Save file | `cmd+s` |
| Command palette | `cmd+p` |
| New file | via file tree |
| Move pane position | via command palette |

### File Tree

- Shows all files in the vault root recursively
- MVP scope: `.md` files only
- Supported operations: create file, rename file, delete file, move file

### Command Palette (`cmd+p`)

- Fuzzy search over files and registered commands
- Commands are named strings resolved via the command registry

---

## Section 7: Terminal Pane

**`src/terminal/`** — embedded pty terminal.

- `portable-pty` for pty spawning
- Spawns `$SHELL`, falls back to `/bin/zsh`
- Renders inside the Vello pipeline via cosmic-text
- Working directory defaults to vault root on spawn

### Terminal Tabs

Multiple independent pty sessions within the terminal pane:

| Action | Keybinding (terminal focused) |
|--------|-------------------------------|
| New terminal tab | `cmd+t` |
| Close terminal tab | `cmd+w` |

Tab bar within the terminal pane shows session names (`zsh 1`, `zsh 2`, etc.).

### MVP Scope

- Text input/output, scrollback buffer
- Copy/paste (`cmd+c` / `cmd+v`)
- Basic ANSI color support

**Out of scope for MVP:** tmux-style splits within the pane, custom shell config UI.

---

## Section 8: Keybindings

**`src/keybindings/`** — one JSON file per OS, loaded at startup based on detected platform.

```
src/keybindings/
├── macos.json     # MVP — fully defined
├── linux.json     # Stub — not shipped in MVP
└── windows.json   # Stub — not shipped in MVP
```

Keybindings map named command strings to key chords. All actions in the app are registered commands — keybindings never call code directly.

Future: `.onyx/keybindings.json` per vault can override defaults.

---

## Section 9: Pluggable Architecture

No plugin loader ships in MVP. The architecture is designed for extensibility from day one.

### Event Bus (`src/shell/`)

All significant actions emit named events:
- `file.opened`, `file.closed`, `buffer.changed`, `cursor.moved`
- `command.executed`, `vault.opened`, `pane.toggled`

Post-MVP plugins subscribe to events to react to editor state.

### Command Registry (`src/shell/`)

All actions are registered as named commands (`file.save`, `pane.terminal.toggle`, etc.). The command palette, keybindings, and future plugins all invoke commands by name — never by direct function call.

### Theme System (`src/render/`)

Themes are named, loadable structs — not hardcoded values. MVP ships one default theme (Zed team colors). Post-MVP, a theme file can be dropped in and registered without code changes.

### Font Config (`src/render/`)

Font family and size are named settings with defaults. Overridable via config without code changes post-MVP.

---

## Section 10: MVP Milestones

Ordered vertical slices — each is independently usable:

| # | Slice | Deliverable |
|---|-------|-------------|
| 1 | **Rendering foundation** | Blank window, GPU context, text renders via Vello/cosmic-text |
| 2 | **Editor core** | Rope buffer + cursor + Vim normal/insert/visual modes on plain text |
| 3 | **WYSIWYG layer** | Markdown AST, inline styles, live preview + raw mode toggle per tab |
| 4 | **Vault system** | `.onyx/config.toml`, open vault = new window, first-launch flow, persisted state |
| 5 | **Workspace shell** | File tree + file ops, tab bar, split panes, command palette, keybinding system, event bus, command registry |
| 6 | **Terminal pane** | Embedded pty, multiple tabs, movable pane position |

**Nice-to-have (MVP stretch):** `cmd+f` in-file search.

**Post-MVP:** encryption, plugin loader, themes, font config UI, Linux/Windows, mobile.
