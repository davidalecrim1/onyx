# Onyx MVP: Playable Editor

## Overview

This document supersedes `2026-02-23-mvp-design.md` for the purposes of the next implementation phase.

The MVP is defined by two user journeys that must work end-to-end. Everything else is deferred.

**Target platform:** macOS only.

---

## User Journeys

### Journey 1 — Create a vault

1. Launch Onyx → welcome screen appears
2. Click "Create Vault" → native folder picker opens
3. Select a folder → Onyx writes `.onyx/config.toml` and `~/.config/onyx/config.toml`
4. Main editor window opens with an empty file tree

### Journey 2 — Open an existing vault and edit a file

1. Launch Onyx → last active vault auto-opens (or welcome screen if none known)
2. File tree shows `.md` files in the vault root
3. Click a file → content loads into the editor, text is visible
4. Edit with Vim keys: Normal mode (`h/j/k/l`, motions), Insert mode (`i`, `a`, typing), `Escape` to return
5. `cmd+s` saves to disk
6. Quit

---

## Architecture

### Data Flow

```
winit KeyboardInput
  → keybindings.rs: chord → command name
  → app.rs: dispatch command OR pass key to VimEngine
  → vim.rs: Key → BufferCommand
  → buffer.rs: apply mutation, mark dirty
  → editor.rs: sync_document() if dirty → Vec<RenderLine>
  → render/mod.rs: layout → rasterize → blit → wgpu present
```

### Ownership Model

```
App
 └── Tab  (single for MVP; Vec<Tab> post-MVP)
      └── Editor
           ├── Buffer  (ropey rope + cursor + selection)
           └── VimEngine  (modal state + yank register)
```

The yank register is scoped to the VimEngine (per-tab). Multi-tab is not interactive in the MVP — the infrastructure exists but the tab bar is not clickable.

### Glyph Pipeline

Text rendering is three distinct stages. All three must complete for characters to appear on screen:

1. **Layout** — `cosmic-text` shapes text runs into glyphs with physical positions and cache keys
2. **Rasterize** — `SwashCache::get_image()` renders each glyph to a pixel buffer
3. **Blit** — the pixel buffer is composited into the Vello scene at the glyph's position using RGBA data (colored glyphs) or a mask + foreground color (subpixel glyphs)

Stage 3 was missing from the previous implementation. It must be completed before any other milestone is attempted.

### HiDPI

- `window.scale_factor()` is read at init and on every `ScaleFactorChanged` event
- All pixel constants (pane widths, row heights, font sizes) are logical pixels multiplied by scale factor before passing to wgpu
- `cosmic-text` receives logical pixels; the wgpu surface operates in physical pixels

---

## Focus Model

```rust
enum Focus { Editor, FileTree, Terminal }
```

| From | Action | To |
|------|--------|----|
| Editor | `cmd+b` (toggle file tree) | FileTree |
| Editor | `cmd+j` (open/focus terminal) | Terminal |
| FileTree | `Enter` (open file) | Editor |
| FileTree | `Escape` | Editor |
| Terminal | `Escape` | Editor |
| Any | focused pane toggled off | Editor |

**Visual indicators:**
- Editor focused: cursor rendered per Vim mode
- FileTree focused: selected row highlighted, editor cursor hidden
- Terminal focused: terminal cursor drawn, editor cursor hidden

**Cursor shape by Vim mode:**

| Mode | Cursor |
|------|--------|
| Normal | Solid block (`█`) |
| Insert | Thin I-beam (`\|`) |
| Visual | Block + selection highlight |

---

## Scroll

- `Buffer` tracks `scroll_offset: usize` — the index of the topmost visible line
- After every cursor move, clamp scroll so the cursor is always within a 3-line margin from the top and bottom of the viewport
- Line-snap only — no smooth scrolling for MVP

---

## Error Handling

All fallible operations return `Result`. No `unwrap()` or `expect()` in production paths.

Errors surface via a status bar at the bottom of the window (single line, auto-clears after 3 seconds).

| Failure | Behavior |
|---------|----------|
| GPU surface creation fails | Log + panic at startup (unrecoverable) |
| Vault directory missing on launch | Welcome screen with "Vault not found" message |
| File open fails | Status bar: "Could not open file" |
| File save fails | Status bar: "Save failed — check disk space or permissions" |
| Config write fails | Log warning, continue (non-fatal) |

---

## Milestones

Each milestone is a vertical slice. The next milestone does not start until the current one passes all acceptance criteria.

### Milestone 1 — Text on screen

Fix the glyph pipeline. No new features.

**Acceptance criteria:**
- `make run` opens a window
- A hardcoded string is visibly rendered on screen
- Characters are crisp on a Retina display (HiDPI scale factor applied correctly)

### Milestone 2 — Editable buffer

Wire keyboard → vim → buffer → renderer.

**Acceptance criteria:**
- Type in Insert mode — characters appear on screen
- `Escape` returns to Normal mode — cursor shape changes to block
- `h/j/k/l` moves the cursor visibly
- Cursor stays within buffer bounds at all times
- Scroll follows cursor (3-line margin, line-snap)

### Milestone 3 — Vault creation

Welcome screen → folder picker → vault written → editor window opens.

**Acceptance criteria:**
- First launch shows welcome screen with "Create Vault" action
- "Create Vault" opens a native folder picker (rfd)
- Selecting a folder writes `.onyx/config.toml` and `~/.config/onyx/config.toml`
- Editor window opens with an empty file tree
- Second launch skips the welcome screen and reopens the vault directly

### Milestone 4 — File tree and open file

File tree renders file names. Clicking one loads the file.

**Acceptance criteria:**
- File tree renders `.md` file names (not only background rectangles)
- Clicking a file loads its content into the buffer
- Content is visible in the editor
- The selected file is highlighted in the tree

### Milestone 5 — Save

`cmd+s` writes the buffer to disk.

**Acceptance criteria:**
- `cmd+s` saves the currently open file
- Save is atomic: write to a temp file, then rename — the original is untouched if the write fails
- A "Saved" indicator appears briefly in the status bar after a successful save

---

## Deferred Features

The following are tracked and scoped for future milestones. None are required for the MVP journeys above.

| Feature | Notes |
|---------|-------|
| Visual mode (`v`, `V`) | VimEngine stubs exist |
| Undo/redo (`u`, `ctrl+r`) | BufferCommand variants exist, apply() is a no-op |
| WYSIWYG live preview | Syntax hiding, cursor-reveals-raw |
| Command palette (`cmd+p`) | Keybinding stub exists, no UI |
| Terminal pane | PTY and grid implemented; character rendering not wired |
| Multiple interactive tabs | Infrastructure exists; tab bar not clickable |
| File tree CRUD | create, rename, delete, move — FileTree methods exist |
| Pane position config | PaneLayout in vault config, never read |
| In-file search (`cmd+f`) | Not started |
| Theme system | Colors hardcoded throughout |
| Plugin loader | Post-MVP |
| Linux / Windows | Post-MVP |

---

## What Was in the Previous Design That Is Now Deferred

The original `2026-02-23-mvp-design.md` included the terminal pane, command palette, pane layout, WYSIWYG rendering, and multi-tab editing as MVP scope. All of these are moved to deferred. The original design also specified incremental Markdown re-parsing ("re-parse only the affected block") — this is downgraded to full re-parse on dirty, incremental is post-MVP.
