# Onyx — Overall Tasks Deferred

## Must Have
- [x] Continue on the file tree and opening markdown files on the UI.
- [x] Basic text editing with cursor, insert/delete, save (Cmd+S), dirty indicator.
- [ ] **Command palette** — Enumerate `Action` variants, display in a fuzzy-searchable overlay, and dispatch through the same `handle_action` path. The `Action` enum in `src/action.rs` is designed for this.

## Nice to Have
- [ ] **Migrate UI layout to Taffy** — Replace manual `Rect` coordinate math (`center_child`, `split_vertical`, `inset`) with Taffy's flexbox layout engine. Components declare sizing constraints and Taffy resolves bounds automatically. Scope: add `taffy` dependency, introduce a layout pass before `paint()`, update all UI components and screens.
