# Onyx

## Architecture

**Input layer** — `winit` delivers OS events to `App`. Raw key events are normalized into chords and resolved to command names via `KeyBindings`. No component below this layer ever receives a raw OS event.

**Dispatch layer** — `CommandRegistry` maps command names to handlers. All user actions — whether from keybindings, the command palette, or plugins — flow through here as named strings (e.g. `file.save`, `pane.file_tree.toggle`). Keybindings never call functions directly.

**Editing layer** — `Editor` owns a `Buffer` and a `VimEngine`. The Vim engine is a pure state machine: it receives a `Key` and returns a `BufferCommand`, never touching shared state. `Editor` applies that command to the buffer. The buffer is rope-backed (`ropey::Rope`) for efficient large-file edits.

**Workspace layer** — Shell operations (vault config, file tree, pane layout) run alongside editing. State changes are broadcast through `EventBus` using dot-namespaced event names (`buffer.changed`, `file.opened`). Components subscribe by name; nothing couples directly.

**Markdown layer** — The buffer's text is parsed into a `Document` AST by `markdown.rs` only when the buffer's dirty flag is set. This AST drives both raw display and the future WYSIWYG rendering pass.

**Render layer** — `Renderer` takes the output of `Editor::build_render_lines()` and the Markdown AST, lays out glyphs with `cosmic-text`, rasterizes them via `SwashCache`, and composites the scene through Vello onto a `wgpu` surface. It knows nothing about editing or Vim state.

## Rust Best Practices

- Never use `unwrap()` or `expect()` in production paths — propagate errors with `?` or handle explicitly with `match`/`if let`
- Never silently discard errors with `let _ =` on fallible operations
- Use full words for variable names — no single-letter abbreviations
- Prefer `src/some_module.rs` over `src/some_module/mod.rs`
- Implement new functionality in existing files unless it is a clearly distinct logical component
- Use variable shadowing in async contexts to scope clones and minimize borrow lifetimes
- Comments explain the *why* only — never summarize what the code does, and never use section separator comments like `// --- Section ---` or `// ── Label ──────`
- Every public method gets exactly one doc comment line (`///`); don't restate the signature — add context, edge cases, or the non-obvious rule being enforced
- Avoid deep nesting — prefer early returns, guard clauses, and extracting nested blocks into named functions. If a block is indented more than 2–3 levels, it's a signal to refactor

## Git Commits

Use conventional commits format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`, `style`

Examples:
- `feat(buffer): add undo/redo stack`
- `fix(vim): handle escape key in insert mode`
- `refactor(render): extract scene builder into helper`

## Test Coverage

- Maintain 80% minimum coverage on core business logic (buffer, vim engine, markdown parser, editor layer, shell commands)
- Write the failing test first — never write implementation code without a failing test that justifies it
- Coverage targets apply to logic modules only; exclude UI rendering, main.rs wiring, and config I/O boilerplate
- Before marking any task complete, run the test suite and confirm no regressions
- Before marking any task complete, run `make format && make lint` and fix any warnings or errors
