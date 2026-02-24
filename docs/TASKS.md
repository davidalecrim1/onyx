# Onyx — Overall Tasks Deferred

## Must Have
- [x] **Replace string-based command registration with typed commands**
  - **Files:** `src/app.rs`, `src/shell/command_registry.rs`
  - The registry uses bare string literals as command identifiers, so typos are silent and the set of valid commands is implicit. Introduce a `Command` enum, replace string keys in `CommandRegistry` with variants, and update `handle_named_command` to match on it. Keybindings JSON still resolves to strings, but the resolution step converts to `Command` before dispatch — unhandled variants become a compile error.

- [x] **Decompose `App::new` into focused initialisation helpers**
  - **File:** `src/app.rs`
  - `App::new` conflates vault detection, tab restoration, cursor restoration, and command registration in one block. Extract each concern into a private method (`load_initial_state`, `register_commands`, etc.) so `App::new` reads as a short, high-level sequence of calls.

## Nice to Have
