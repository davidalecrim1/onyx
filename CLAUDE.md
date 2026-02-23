# Onyx

## Rust Best Practices

- Never use `unwrap()` or `expect()` in production paths — propagate errors with `?` or handle explicitly with `match`/`if let`
- Never silently discard errors with `let _ =` on fallible operations
- Use full words for variable names — no single-letter abbreviations
- Prefer `src/some_module.rs` over `src/some_module/mod.rs`
- Implement new functionality in existing files unless it is a clearly distinct logical component
- Use variable shadowing in async contexts to scope clones and minimize borrow lifetimes
- Comments explain the *why* only — never summarize what the code does

## Test Coverage

- Maintain 80% minimum coverage on core business logic (buffer, vim engine, markdown parser, editor layer, shell commands)
- Write the failing test first — never write implementation code without a failing test that justifies it
- Coverage targets apply to logic modules only; exclude UI rendering, main.rs wiring, and config I/O boilerplate
- Before marking any task complete, run the test suite and confirm no regressions
