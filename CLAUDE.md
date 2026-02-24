# Onyx

## Project Structure

```
src/
├── main.rs          # Entry point — env_logger init, winit event loop, nothing else
├── app.rs           # App struct, ApplicationHandler impl, window + renderer lifecycle
├── render.rs        # GPU pipeline: wgpu surface + Vello scene + cosmic-text layout
├── buffer.rs        # Rope-backed text buffer with cursor, selection, insert/delete/move
├── vim.rs           # Vim modal state machine — receives Key, emits BufferCommand
├── editor.rs        # Editor layer tying buffer + vim; RenderLine/RenderSpan output
├── markdown.rs      # Markdown parser — Document, Block, Inline AST types
├── terminal.rs      # Embedded terminal: pty spawn, VTE parsing, grid cells
└── shell/           # Workspace infrastructure (only subdir — multiple tightly-coupled files)
    ├── mod.rs
    ├── vault.rs             # VaultConfig, GlobalConfig, TabState serialization
    ├── command_registry.rs  # Named commands → closures dispatch
    ├── event_bus.rs         # Pub/sub for named events (dot notation: buffer.changed)
    ├── keybindings.rs       # JSON chord → command name resolution
    └── file_tree.rs         # .md file listing, CRUD within vault root
```

### Architecture Layers

```
winit events → KeyBindings → CommandRegistry → VimEngine → BufferCommand → Buffer
                                                              ↓
                                              Editor::build_render_lines()
                                                              ↓
                                              Renderer (Vello + cosmic-text → wgpu)
```

### Key Patterns

- **Command Registry**: All user actions are named commands (e.g. `file.save`, `pane.file_tree.toggle`). Keybindings resolve to command names, never direct function calls.
- **Event Bus**: State changes emit named events (`buffer.changed`, `file.opened`). Decouples components.
- **Pure Vim State Machine**: VimEngine only receives Key and returns BufferCommand. Never touches the buffer directly.
- **Dirty-flag lazy re-parse**: Markdown AST only re-parsed when buffer is marked dirty, not every frame.
- **Rope-backed buffer**: Uses `ropey::Rope` for efficient large-file editing.

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
