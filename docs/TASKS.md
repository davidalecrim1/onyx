# Onyx — Overall Tasks Deferred

## Must Have
- [ ] Continue on the file tree and opening markdown files on the UI.

## Nice to Have
- [ ] **Migrate UI layout to Taffy** — Replace manual `Rect` coordinate math (`center_child`, `split_vertical`, `inset`) with Taffy's flexbox layout engine. Components declare sizing constraints and Taffy resolves bounds automatically. Scope: add `taffy` dependency, introduce a layout pass before `paint()`, update all UI components and screens.
