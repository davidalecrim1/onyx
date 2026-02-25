# Vision

An Obsidian like Markdown editor built in Rust. Inspired by Zed minimalist and futuristic approach with a amazing performance and low memory usage.

## Philosophy
- Truelly open source.
- Allow any open source plugin to extend it's functionality.

### Basic Features
- Native app installed on the OS like Obsidian/Zed.
- Everything build in Rust.
- Optimized for high performance.
- No managed cloud sync, relies on the ICloud, OneDrive managed file sync and storage.
- Optimized for key bindings and mouseless by design (like Zed).
- Very similar key bindings like Zed to be friction less to use by developers.
- Initially default Zed team colors inspired.
- Markdown by design. Allow to preview rendered files and native (raw).

### Possible Features 
- Native encryption of files with password using something like AES 256.
- Native terminal integration to support running Claude Code on the terminal to leverage AI on the files.
- An integrated Vim experience for editing files.
- An alternative to support Git as well for native file sync by design.
  - Reflect on this based on how the Git extensions works for Obsidian.
- Audio notes support with transcribe feature.
- A native app for mobile to leverage the files on ICloud.

## Initially Techonology Stack
| Layer | Tool | 
| ----------- | ----------- | 
| Windowing | winit |
| GPU | wgpu | 
| Rendering | Vello |
| Text layout | cosmic-text |
| Layout | taffy |
