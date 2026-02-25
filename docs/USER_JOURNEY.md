# Onyx - User Journeys Definitions  

## User Journeys

### Journey 1 — Create a vault

1. Launch Onyx → welcome screen appears
2. Click "Create Vault" → native folder picker opens
3. Select a folder → Onyx writes `.onyx/config.toml` and `~/.config/onyx/config.toml`
4. Main editor window opens with an empty file tree

### Journey 2 — Open an existing vault and edit a file

1. Launch Onyx → last active vault auto-opens (or welcome screen if none known)
2. File tree shows `.md` files in the vault root
3. Click a file → content loads into the editor, text is visible
4. Edit with Vim keys: Normal mode (`h/j/k/l`, motions), Insert mode (`i`, `a`, typing), `Escape` to return
5. `cmd+s` saves to disk
6. Quit
