# Plan: Zed-Inspired File Tree UI Redesign

## Context

The current UI uses a purple-tinted dark theme with tight spacing, no hover states, and basic text-only rendering for the file tree. The goal is to adopt Zed's gray-based, minimalist aesthetic — starting with the file tree sidebar — to make the app feel clean, professional, and easy to use. The VISION.md already states "Initially default Zed team colors inspired", so this aligns with project intent.

## Step 1: Update Theme to Zed's Gray Palette

**File:** `src/ui/theme.rs`

Add three new color fields and update all values to match Zed's One Dark:

```rust
pub struct Theme {
    // existing fields
    pub background: Color,
    pub surface: Color,
    pub separator: Color,
    pub accent: Color,
    pub accent_dim: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub typography: Typography,
    // new fields
    pub surface_hover: Color,   // subtle hover feedback
    pub surface_active: Color,  // selected/active items
    pub border: Color,          // panel borders (distinct from separator)
}
```

New `dark()` values (Zed One Dark palette):

| Token | Current | New (Zed) |
|-------|---------|-----------|
| background | `rgb(28,28,32)` | `#282c33` |
| surface | `rgb(32,32,38)` | `#2f343e` |
| separator | `rgb(50,50,60)` | `#464b57` |
| accent | `rgb(138,92,246)` purple | `#74ade8` blue |
| accent_dim | `rgb(80,56,150)` | `#5683b0` dimmed blue |
| text_primary | `rgb(230,230,230)` | `#dce0e5` |
| text_secondary | `rgb(140,140,155)` | `#a9afbc` |
| surface_hover | — | `#363c46` |
| surface_active | — | `#454a56` |
| border | — | `#363c46` |

Typography adjustments: `body_size` 18 -> 16, `small_size` 16 -> 14, `line_height_factor` 1.2 -> 1.4.

## Step 2: Add Cursor Position to DrawContext

**File:** `src/ui/canvas.rs`

Add `cursor_position: (f32, f32)` field to `DrawContext`. This enables hover detection during paint without adding state to individual components.

**File:** `src/app.rs`

Pass `self.cursor_position` (already tracked) into `DrawContext` at both render call sites.

## Step 3: Restyle File Tree Sidebar

**File:** `src/editor_view.rs`

### Constants
| Constant | Current | New |
|----------|---------|-----|
| `SIDEBAR_WIDTH` | 220 | 240 |
| `ROW_HEIGHT` | 22 | 28 |
| `TAB_BAR_HEIGHT` | 32 | 36 |
| indent per depth | 16 | 20 |

Add named constants: `SIDEBAR_PADDING_LEFT` (12), `INDENT_PER_DEPTH` (20), `HEADER_FONT_SIZE` (12), `HEADER_HEIGHT` (32).

### Vault Name Header
Replace the large vault name with a small uppercase section label (12px, `text_secondary`), matching Zed's "PROJECT" header style.

### File Tree Rows
- **Hover state:** Compare `ctx.cursor_position` against each row rect. Draw `surface_hover` background on hover.
- **Selection:** Use `surface_active` instead of `accent_dim`.
- **Files:** Remove bullet prefix — just indented text. Cleaner.
- **Directories:** Keep chevrons (`▸`/`▾`) but render in `text_secondary` color. Draw directory name separately after chevron.
- **Vertical centering:** Center text within `ROW_HEIGHT` using `(ROW_HEIGHT - font_size) / 2.0`.

### Tab Bar
- Remove accent underline on active tab — rely on background contrast only (active = `background`, inactive = `surface`).
- Add hover state for inactive tabs (`surface_hover`).
- Use `border` color for the bottom separator line.

### Content Area
Increase padding: left 12 -> 16, top 16 -> 20 (4px grid alignment).

## Step 4: Update Documentation

**File:** `docs/UI_ARCHITECTURE.md`
- Update the `Theme` struct listing to include the three new fields (`surface_hover`, `surface_active`, `border`).
- Update the "What we intentionally skip" section — remove the "Hover/focus states" bullet since hover is now implemented.
- Update the `DrawContext` description to mention `cursor_position`.

**File:** `CLAUDE.md` — No changes needed (contains coding standards, not UI specifics).

## Verification

1. `cargo build` — confirm all new Theme fields compile and are wired through
2. `cargo test` — existing tests pass (they test logic, not rendering)
3. `make format && make lint` — no warnings
4. Run the app manually:
   - Welcome screen renders correctly with new blue accent and gray palette
   - File tree sidebar shows hover highlights on mouse movement
   - Selected file shows `surface_active` background
   - Directory chevrons render in secondary text color
   - Tab bar shows hover/active states correctly
   - Overall feel matches Zed's clean gray aesthetic

## Critical Files
- `src/ui/theme.rs` — palette and new color tokens
- `src/ui/canvas.rs` — `DrawContext` cursor_position field
- `src/app.rs` — wire cursor_position into DrawContext
- `src/editor_view.rs` — all visual changes (sidebar, file tree, tabs)
- `docs/UI_ARCHITECTURE.md` — update Theme struct and hover docs
