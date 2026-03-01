# UI Architecture

Onyx's frontend is a React + TypeScript SPA served by a Tauri 2.0 WebView. Components communicate with the Rust backend via Tauri IPC (`invoke`). Styling uses Tailwind CSS with the One Dark palette.

## Component Hierarchy

```
App.tsx                     — router: WelcomePage | EditorPage
├── WelcomePage.tsx         — "Create Vault" + "Open Vault" buttons
│                             calls open() from tauri-plugin-dialog
│                             then invoke("create_vault" | "open_vault")
└── EditorPage.tsx          — main editor shell, owns all editor state
    ├── FileTree.tsx        — recursive tree, invoke("get_file_tree")
    ├── TabBar.tsx          — open tabs, active tab switching, close
    └── MarkdownEditor.tsx  — CodeMirror 6 + vim mode + markdown lang
```

## State Management

React `useState` + `useReducer` at `EditorPage` level — no external state library. Vault path lives in `App`-level state and is passed down as props.

`EditorPage` uses a reducer with these actions:
- `open_file` — add tab and cache content
- `close_tab` — remove tab, clean up content cache
- `activate_tab` — switch active tab
- `update_content` — mark file dirty on edit
- `mark_saved` — clear dirty flag after successful write

## Theming

All colors defined in `tailwind.config.js` as custom tokens mapped from the One Dark palette:

| Token | Hex |
|-------|-----|
| `background` | `#282c33` |
| `surface` | `#2f343e` |
| `surface-hover` | `#363c46` |
| `surface-active` | `#454a56` |
| `accent` | `#74ade8` |
| `text-primary` | `#dce0e5` |
| `text-secondary` | `#a9afbc` |

## IPC Boundary

All backend calls go through `invoke` from `@tauri-apps/api/core`. The TypeScript side treats every command as `Result<T, String>` — success returns the typed DTO, failure throws a string error.

| Command | Args | Returns |
|---------|------|---------|
| `create_vault` | `path: string` | `VaultInfo` |
| `open_vault` | `path: string` | `VaultInfo` |
| `get_file_tree` | `vault_path: string` | `FileTreeEntryDto[]` |
| `read_file` | `path: string` | `string` |
| `write_file` | `path: string, content: string` | `void` |
| `get_known_vaults` | — | `VaultEntry[]` |
