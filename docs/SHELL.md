# Shell Architecture

The `shell/` module is the workspace infrastructure layer — everything that sits between raw OS events and the editor core.

## Components

### CommandRegistry
All user-visible actions are registered as named commands (`file.save`, `pane.file_tree.toggle`). Keybindings resolve to command names; nothing in the UI calls editor logic directly.

```
KeyBindings → command name → CommandRegistry::execute()
```

This keeps the input layer decoupled from behavior. Adding a new action means registering a command name — the keybinding is separate.

### EventBus
State changes are broadcast as named events (`buffer.changed`, `file.opened`, `pane.toggled`). Components subscribe to event names; the emitter doesn't know who's listening.

Use dot notation for event names: `<domain>.<action>`.

### KeyBindings
Loads a platform-specific JSON file (`keybindings/macos.json`) at compile time via `include_str!`. Maps chord strings like `"cmd+s"` to command names. Chords are built from modifier state + the logical key character, joined with `+`.

```json
{
  "cmd+s":        "file.save",
  "cmd+b":        "pane.file_tree.toggle"
}
```

### FileTree
Scans the vault root for `.md` files and directories (hidden entries excluded). Provides create, rename, delete, and move operations — all paths are relative to the vault root.

### VaultConfig / GlobalConfig
Persistence layer. `GlobalConfig` tracks which vaults have been opened and which was last active. `VaultConfig` stores per-vault state: open tabs, cursor positions, view modes.

## Data Flow

```
winit KeyboardInput
  └─ build_chord()
       └─ KeyBindings::resolve()  →  command name
            └─ App::handle_named_command()
                 ├─ direct: file.save, pane.file_tree.toggle
                 └─ fallback: CommandRegistry::execute()
                                   └─ EventBus::emit()
```

## Conventions

- Command names use dot-separated namespaces: `<domain>.<action>` or `<domain>.<subdomain>.<action>`.
- Event names follow the same pattern.
- `handle_named_command` handles commands that need direct `App` state access (toggling `file_tree_visible`, calling `save_vault_state`). Everything else goes through the registry.
