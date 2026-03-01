# Plan: Unified button colors, Zed Mono font, per-vault font config

## Context

The welcome screen buttons ("Create vault" / "Open vault") use different colors (accent vs accent_dim). They should match. The app uses system default fonts instead of a purposeful choice. The user wants Zed Mono bundled as the default font, per-vault font configuration (family, sizes), and larger default font sizes.

## Changes

### 1. Bundle Zed Mono font

- Download Zed Mono Extended (Regular) `.ttf` from Zed's GitHub releases (SIL OFL license)
- Place at `assets/fonts/zed-mono-extended-regular.ttf`
- Add a `LICENSES-THIRD-PARTY.md` or note in README for font attribution
- In `text.rs`: load the embedded font via `include_bytes!` into `FontSystem` at init time using `db_mut().load_font_data()`
- Pass `Attrs::new().family(Family::Name("Zed Mono Extended"))` in `draw_text` and `measure_text` instead of bare `Attrs::new()`

**Files:** `src/text.rs`, new `assets/fonts/` directory

### 2. Per-vault font configuration

Extend `VaultConfig` with optional font settings that override defaults:

```toml
name = "my-vault"

[editor]
font_family = "Zed Mono Extended"
font_size = 18.0
ui_font_size = 15.0
```

- Add `EditorConfig` struct (optional fields with `#[serde(default)]`) inside `vault_config.rs`
- `VaultConfig` gets `pub editor: EditorConfig`
- `EditorConfig` has: `font_family: Option<String>`, `font_size: Option<f32>`, `ui_font_size: Option<f32>`
- Defaults: font_family = "Zed Mono Extended", font_size = 18.0, ui_font_size = 15.0
- When a vault opens, merge its config with defaults to build Typography
- Add a `resolve_typography()` method that produces `Typography` from `EditorConfig` + defaults

**Files:** `src/vault_config.rs`

### 3. Increase default font sizes

Update `Typography` defaults in `theme.rs`:

| Token | Old | New |
|-------|-----|-----|
| `title_size` | 48.0 | 48.0 (unchanged) |
| `body_size` | 16.0 | 18.0 |
| `small_size` | 14.0 | 15.0 |

**Files:** `src/ui/theme.rs`

### 4. Wire vault config into typography

- In `app.rs`, when opening/creating a vault, read `VaultConfig.editor` and update `self.theme.typography` accordingly
- Add `Theme::with_editor_config(&mut self, config: &EditorConfig)` to apply vault overrides

**Files:** `src/app.rs`, `src/ui/theme.rs`

### 5. Unify button colors

Both welcome screen buttons should use the same accent color. Remove the `.accent(true)` distinction -- both buttons get `accent` color.

- In `welcome.rs`: remove `.accent(true)` from "Create vault" so both use the same style
- In `button.rs`: remove the `accent` field entirely; always use `ctx.theme.accent` as fill

**Files:** `src/welcome.rs`, `src/ui/button.rs`

### 6. Pass font family through text system

- Add a `default_family: String` field to `TextSystem`
- `draw_text` and `measure_text` use `Attrs::new().family(Family::Name(&text_system.default_family))`
- `TextSystem::new()` takes the font family name as parameter (or uses default)

**Files:** `src/text.rs`, `src/app.rs`

### 7. Update documentation

- `docs/MVP.md`: Add font configuration section under Vault Config
- `docs/UI_ARCHITECTURE.md`: Update Theme struct docs to mention Typography is vault-configurable
- Update existing vault config description to mention `[editor]` section

**Files:** `docs/MVP.md`, `docs/UI_ARCHITECTURE.md`

### 8. Tests

- Update `vault_config.rs` tests for new `EditorConfig` field (backward compat with existing configs)
- Ensure `text.rs` tests still pass with explicit font family
- Update `welcome.rs` tests if button API changes

## File summary

| File | Change |
|------|--------|
| `assets/fonts/zed-mono-extended-regular.ttf` | New -- bundled font |
| `src/text.rs` | Load embedded font, accept family name |
| `src/vault_config.rs` | Add `EditorConfig` with font settings |
| `src/ui/theme.rs` | Increase default sizes, add vault config override |
| `src/ui/button.rs` | Remove accent/dim distinction |
| `src/welcome.rs` | Both buttons same style |
| `src/app.rs` | Wire vault config -> theme + text system |
| `docs/MVP.md` | Document font config |
| `docs/UI_ARCHITECTURE.md` | Update theme docs |

## Verification

1. `cargo build` -- compiles without error
2. `cargo test` -- all tests pass
3. `make format && make lint` -- clean
4. Run the app -- welcome screen shows both buttons in same blue accent color
5. Open a vault -- text renders in Zed Mono Extended at larger sizes
6. Edit `<vault>/.onyx/config.toml` with custom `[editor]` section -- font changes apply on next open
