# Onyx MVP Design

## Overview

Onyx is a native Markdown editor built with Tauri 2.0, React, and TypeScript. Inspired by Zed's minimalist philosophy, it targets developers who want an Obsidian-like experience with Vim-first editing and mouseless navigation.

**Non-goals for MVP:** encryption, plugin loader, themes, mobile, Git sync.

## Section 1: Project Structure

Tauri monorepo — Rust backend in `src-tauri/`, React frontend in `src/`.

**Data flow:**

```
User action (React UI)
  → invoke("command_name", args)    [Tauri IPC]
  → src-tauri/src/commands.rs       [Rust handler]
  → vault / file_tree / global_config logic
  → Result<T, String> → JSON → React state update
  → Re-render
```

---

## Section 2: Technology Stack

| Layer | Tool |
|-------|------|
| App shell | Tauri 2.0 |
| Frontend | React 18 + TypeScript |
| Bundler | Vite 5 |
| Styling | Tailwind CSS (One Dark palette) |
| Editor | CodeMirror 6 + vim mode |
| Config serialization | serde + toml |

---

## Section 3: Vault System

A vault is a directory opened in Onyx. Each vault has its own file tree, tabs, and persisted state.

**Vault config:** `.onyx/config.toml` inside the vault root.

**Global config:** `~/.config/onyx/config.toml` stores known vaults and last active vault.

### First Launch

1. Welcome screen — two buttons: **Create Vault** and **Open Vault**
2. Both open a native folder picker via `tauri-plugin-dialog`
3. On selection, backend creates `.onyx/config.toml` and registers the vault in global config
4. Frontend transitions to `EditorPage` with the vault path

### Subsequent Launches

1. Onyx reads `~/.config/onyx/config.toml`
2. Reopens last active vault (future: restore open tabs and cursor positions)

---

## Section 4: Editor Core

- CodeMirror 6 as the editor engine
- `@codemirror/lang-markdown` for Markdown language support
- `@replit/codemirror-vim` for Vim keybindings
- Files are loaded via `read_file` IPC command and saved via `write_file`
- Cmd/Ctrl+S triggers save

---

## Section 5: File Tree

- Recursive scan via `scan_file_tree` Rust function
- Recognized extensions: `.md`, `.canvas`, `.pdf`, images, audio, video
- Dot-directories excluded (e.g. `.onyx`, `.git`)
- Sorted: directories first, then alphabetically within each group
- Collapsible directories, active file highlighted
