# Onyx

A keyboard-first Markdown editor built in Rust. Inspired by Zed's minimalist aesthetic, designed for performance and low memory usage.

## Features

- Native app — no Electron, no web runtime
- Vim keybindings and mouseless workflow
- Split panes, configurable layout
- Integrated terminal
- File encryption (AES-256)
- Syncs via iCloud or OneDrive — no proprietary cloud

## Stack

| Layer | Crate |
|---|---|
| Windowing | winit |
| GPU | wgpu |
| Rendering | vello |
| Text layout | cosmic-text |
| Layout | taffy |

## License

MIT
