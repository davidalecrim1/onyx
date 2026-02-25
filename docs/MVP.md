# Onyx MVP Design

## Overview

Onyx is a native, GPU-rendered Markdown editor built entirely in Rust. Inspired by Zed's minimalist and performance-first philosophy, it targets developers who want an Obsidian-like experience with Vim-first editing, mouseless navigation, and a pluggable architecture.

**Non-goals for MVP:** encryption, plugin loader, themes, mobile, Git sync.

## Section 1: Project Structure

Single Rust workspace using `cargo init .` convention. Each crate has a focused responsibility.

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

## Section 4: Editor Core

### Buffer (`src/buffer/`)

- Rope data structure via `ropey` for efficient insert/delete on large files
- One buffer per open file, owned by the vault session
- Cursor and selection state lives on the buffer


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

### File Tree

- Shows all files in the vault root recursively
- MVP scope: `.md` files only
- Supported operations: create file, rename file, delete file, move file
