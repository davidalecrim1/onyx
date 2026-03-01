# Onyx

A keyboard-first Markdown editor. Inspired by Zed's minimalist aesthetic — Vim-first editing, mouseless workflow, syncs via iCloud or OneDrive with no proprietary cloud.

## Stack

| Layer | Tool |
|---|---|
| App shell | Tauri 2.0 |
| Frontend | React 18 + TypeScript |
| Bundler | Vite 5 |
| Styling | Tailwind CSS |
| Editor | CodeMirror 6 + vim mode |
| Backend | Rust (serde, toml, dirs-next) |

## Development

```sh
make install   # npm install
make dev       # tauri dev (hot reload)
make test      # cargo test in src-tauri/
make lint      # clippy + eslint
make build     # production build
```

## License

MIT
