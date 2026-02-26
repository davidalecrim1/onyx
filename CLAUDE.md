# Onyx

# Rust Best Practices

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

## General Guidance

- Maintain 80% minimum coverage on core business logic
- Write the failing test first — never write implementation code without a failing test that justifies it
- Before marking any task complete, run the test suite and confirm no regressions
- Before marking any task complete, run `make format && make lint` and fix any warnings or errors
