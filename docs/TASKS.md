# Onyx — Overall Tasks Deferred

## Must Have
- [ ] Continue on the file tree and opening markdown files on the UI.

## Experiments
- [ ] **Spike: Tauri 2.0 migration** — Stand up a parallel Tauri + React + TypeScript branch to validate the stack. Scope: scaffold a Tauri 2.0 project, port `vault.rs`, `vault_config.rs`, and `global_config.rs` as Tauri commands, wire up a basic file tree in React, and integrate CodeMirror 6 with `@codemirror/vim` for the editor. Goal is to assess build velocity, feel, and whether the WebView rendering is acceptable for the editing experience before committing.

## Nice to Have
- [ ] **Migrate UI layout to Taffy** — Replace manual `Rect` coordinate math (`center_child`, `split_vertical`, `inset`) with Taffy's flexbox layout engine. Components declare sizing constraints and Taffy resolves bounds automatically. Scope: add `taffy` dependency, introduce a layout pass before `paint()`, update all UI components and screens.
