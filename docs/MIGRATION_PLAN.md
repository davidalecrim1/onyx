# Plan: Migrate Onyx to Tauri 2.0 + React + TypeScript

## Context

Onyx currently uses a custom GPU rendering pipeline (winit + wgpu + vello + cosmic-text) which gives low-level control but makes UI iteration painful — every new interactive element requires deep rendering knowledge. The user wants to pay the Tauri overhead in exchange for React's component ecosystem and fast UI iteration.

This migration was already anticipated in `docs/TASKS.md` as a planned spike. The Rust business logic (vault, config, file tree) is clean and reusable as-is; only the rendering/UI code gets replaced.

**Outcome:** Same app goal and workflow (Obsidian-like Markdown editor), new rendering model: Tauri WebView shell with React+TypeScript frontend, Rust backend exposed via Tauri IPC commands.

---

## New Project Structure

```
onyx/
├── src/                        # React + TypeScript frontend (replaces old src/)
│   ├── main.tsx
│   ├── App.tsx
│   ├── pages/
│   │   ├── WelcomePage.tsx
│   │   └── EditorPage.tsx
│   └── components/
│       ├── FileTree.tsx
│       ├── TabBar.tsx
│       └── MarkdownEditor.tsx
├── src-tauri/                  # Tauri Rust backend (new)
│   ├── src/
│   │   ├── main.rs             # Tauri entry point (replaces winit event loop)
│   │   ├── commands.rs         # #[tauri::command] IPC handlers
│   │   ├── vault.rs            # moved from src/vault.rs (unchanged)
│   │   ├── vault_config.rs     # moved from src/vault_config.rs (unchanged)
│   │   ├── global_config.rs    # moved from src/global_config.rs (unchanged)
│   │   ├── file_tree.rs        # moved from src/file_tree.rs (unchanged)
│   │   └── error.rs            # moved from src/error.rs (unchanged)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── Makefile                    # updated
└── README.md                   # updated
```

---

## Files to Delete

These are replaced by the Tauri + React equivalents:

- `src/main.rs` — winit event loop, replaced by Tauri entry point
- `src/app.rs` — app state/event routing, replaced by React state
- `src/welcome.rs` — GPU-painted welcome screen, replaced by WelcomePage.tsx
- `src/editor_view.rs` — GPU-painted editor, replaced by EditorPage.tsx
- `src/gpu.rs` — wgpu/vello renderer, no longer needed
- `src/text.rs` — cosmic-text integration, no longer needed
- `src/ui.rs` — custom component module exports
- `src/ui/` — entire directory (theme, canvas, button, label, panel, hit_test, rect)
- `docs/WINIT.md` — winit-specific internals doc, no longer relevant

---

## Files to Move (unchanged content)

- `src/vault.rs` → `src-tauri/src/vault.rs`
- `src/vault_config.rs` → `src-tauri/src/vault_config.rs`
- `src/global_config.rs` → `src-tauri/src/global_config.rs`
- `src/file_tree.rs` → `src-tauri/src/file_tree.rs`
- `src/error.rs` → `src-tauri/src/error.rs`
- `Cargo.toml` → `src-tauri/Cargo.toml` (with dependencies rewritten for Tauri)
- `Cargo.lock` → `src-tauri/Cargo.lock`
- `.clippy.toml` → `src-tauri/.clippy.toml`

---

## Backend: src-tauri/Cargo.toml

Remove all GPU/rendering deps, add Tauri:

```toml
[package]
name = "onyx"
version = "0.1.0"
edition = "2021"

[lib]
name = "onyx_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
dirs-next = "2"
log = "0.4"
env_logger = "0.11"

[dev-dependencies]
tempfile = "3"

[profile.dev]
opt-level = 1
```

---

## Backend: src-tauri/src/main.rs

Standard Tauri 2.0 entry point that registers all commands:

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::create_vault,
            commands::open_vault,
            commands::get_file_tree,
            commands::read_file,
            commands::write_file,
            commands::get_known_vaults,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
```

---

## Backend: src-tauri/src/commands.rs

New file exposing business logic via Tauri IPC. All commands serialize to JSON for the frontend.

```rust
#[tauri::command]
pub fn create_vault(path: String) -> Result<VaultInfo, String> { ... }

#[tauri::command]
pub fn open_vault(path: String) -> Result<VaultInfo, String> { ... }

#[tauri::command]
pub fn get_file_tree(vault_path: String) -> Result<Vec<FileTreeEntryDto>, String> { ... }

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> { ... }

#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> { ... }

#[tauri::command]
pub fn get_known_vaults() -> Result<Vec<VaultEntry>, String> { ... }
```

Note: `rfd` is removed; folder picker uses `tauri-plugin-dialog` from the frontend via `open()`.

DTOs (serializable structs) needed:
- `VaultInfo { name: String, root: String }`
- `FileTreeEntryDto { name: String, path: String, is_directory: bool, depth: usize, children: Vec<...> }`
- `VaultEntry { name: String, path: String }` (already in `global_config.rs`)

---

## Frontend: package.json dependencies

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "react": "^18",
    "react-dom": "^18",
    "@codemirror/view": "^6",
    "@codemirror/state": "^6",
    "@codemirror/lang-markdown": "^6",
    "@uiw/react-codemirror": "^4",
    "@replit/codemirror-vim": "^6"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "^4",
    "typescript": "^5",
    "tailwindcss": "^3",
    "vite": "^5",
    "@tauri-apps/cli": "^2"
  },
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "tauri": "tauri"
  }
}
```

---

## Frontend: Component Hierarchy

```
App.tsx                 — router: WelcomePage | EditorPage
├── WelcomePage.tsx     — "Create Vault" + "Open Vault" buttons
│                         calls open() from tauri-plugin-dialog
│                         then invoke("create_vault" | "open_vault")
└── EditorPage.tsx      — main editor shell
    ├── FileTree.tsx    — recursive tree, invoke("get_file_tree")
    ├── TabBar.tsx      — open tabs, active tab switching
    └── MarkdownEditor.tsx — CodeMirror 6 + vim mode + markdown lang
```

State management: React `useState` + `useReducer` at `EditorPage` level (no external state library for MVP). Vault path stored in `App`-level state and passed down.

---

## Styling

Use **Tailwind CSS** — gives the fastest iteration for a dark-themed editor UI. The One Dark palette from the current `theme.rs` maps directly to Tailwind config colors:

```js
// tailwind.config.js
colors: {
  background: '#282c33',
  surface: '#2f343e',
  'surface-hover': '#363c46',
  'surface-active': '#454a56',
  accent: '#74ade8',
  'text-primary': '#dce0e5',
  'text-secondary': '#a9afbc',
}
```

---

## src-tauri/tauri.conf.json

```json
{
  "productName": "Onyx",
  "version": "0.1.0",
  "identifier": "com.onyx.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420"
  },
  "app": {
    "windows": [{
      "title": "Onyx",
      "width": 1200,
      "height": 800,
      "minWidth": 800,
      "minHeight": 600
    }]
  }
}
```

---

## Makefile Updates

```makefile
.PHONY: install dev build test lint check format clean

install:
	npm install

dev:
	npm run tauri dev

build:
	npm run tauri build

test:
	cd src-tauri && cargo test

lint:
	cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
	npx eslint src/

check:
	cd src-tauri && cargo check --all-targets

format:
	cd src-tauri && cargo fmt
	npx prettier --write src/

clean:
	cd src-tauri && cargo clean
	rm -rf dist node_modules
```

---

## Documentation Updates

| File | Change |
|------|--------|
| `README.md` | Replace stack table with Tauri/React/TypeScript/Vite/CodeMirror |
| `docs/MVP.md` | Update "Technology Stack" section and data flow (winit → Tauri IPC) |
| `docs/UI_ARCHITECTURE.md` | Rewrite to describe React component hierarchy and Tailwind theming |
| `docs/VISION.md` | Update "Initially Technology Stack" table |
| `docs/TASKS.md` | Mark Tauri spike complete; add new frontend tasks |
| `docs/WINIT.md` | Delete (winit-specific, no longer relevant) |
| `docs/ROPE.md` | Keep as-is (still a future backend concern) |
| `docs/USER_JOURNEY.md` | Keep as-is (user journey unchanged) |

---

## Implementation Order

1. Verify on branch `claude/migrate-to-tauri-VCD7h`
2. Create `src-tauri/` directory tree; move/adapt business logic files
3. Write `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`
4. Write `src-tauri/src/main.rs` and `src-tauri/src/commands.rs`
5. Delete old `src/` UI/GPU files and root `Cargo.toml`/`Cargo.lock`
6. Write `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`
7. Write React frontend: `App.tsx`, `WelcomePage.tsx`, `EditorPage.tsx`, `FileTree.tsx`, `TabBar.tsx`, `MarkdownEditor.tsx`
8. Configure Tailwind CSS with the One Dark palette
9. Update `Makefile`
10. Update `README.md` and all affected `docs/`
11. Run `make install && make test && make lint`
12. Commit and push to branch

---

## Verification

- `make test` — all existing backend unit tests pass (vault, file_tree, vault_config, global_config)
- `make lint` — no clippy warnings in Rust; no TS errors
- `make dev` — Tauri dev window opens with the welcome screen
- Create vault flow: click "Create Vault" → folder dialog → vault created → editor opens with file tree
- Open vault flow: click "Open Vault" → folder dialog → file tree populated → click a `.md` file → CodeMirror loads content with vim mode active
- `make format` — no formatting changes needed (already clean)
