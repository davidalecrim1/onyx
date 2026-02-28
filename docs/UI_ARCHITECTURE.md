# UI Architecture

Onyx's UI layer is a lightweight component system inspired by Zed's GPUI. It avoids traits, layout engines, and vtable dispatch in favor of builder-pattern structs with `paint(bounds)` methods.

## Core principle: bounds flow down

A component never knows the window size. It receives a `Rect` (its allocated bounds) from its parent and paints within it. The app constructs the root bounds from the window dimensions and passes it to the active screen; the screen subdivides it and passes sub-rects to child components.

```
App (owns window size)
 └─ constructs Rect(0, 0, logical_w, logical_h)
     └─ WelcomeScreen.paint(bounds)
         ├─ Label.paint(title_bounds)
         ├─ Button.paint(create_bounds)
         └─ Button.paint(open_bounds)
```

## Anatomy of a component

Every component follows the same shape:

```rust
pub struct MyComponent {
    // required fields
    label: String,
    bounds: Rect,
    // optional builder fields with defaults
    accent: bool,
}

impl MyComponent {
    /// Constructor with required fields.
    pub fn new(label: &str, bounds: Rect) -> Self { ... }

    /// Builder method for optional configuration.
    pub fn accent(mut self, value: bool) -> Self {
        self.accent = value;
        self
    }

    /// Paints into the scene using the provided draw context.
    pub fn paint(self, ctx: &mut DrawContext, hits: &mut HitSink) {
        // use self.bounds for all positioning
        // use ctx.theme for colors and typography
        // use ctx.scene for draw calls
        // register clickable regions with hits.push(id, self.bounds)
    }
}
```

Key rules:
- `paint()` consumes `self` — a component is built, configured, and painted once per frame.
- `bounds: Rect` is always a required parameter, passed in by the parent.
- `DrawContext` bundles `scene + text_system + theme + cursor_position` so paint signatures stay small.
- Interactive components take `&mut HitSink` and register their clickable region.

## Screens as components

Screens (WelcomeScreen, EditorView) follow the same pattern as leaf components. They receive a `Rect` from the app and subdivide it for their children:

```rust
impl WelcomeScreen {
    pub fn paint(&self, ctx: &mut DrawContext, hits: &mut HitSink, bounds: Rect) {
        // center content cluster within bounds
        let content = bounds.center_child(content_width, content_height);

        // subdivide for children
        let title_bounds = Rect::new(content.x, content.y, content.width, title_height);
        Label::new("Onyx", title_size, color)
            .align(Align::Center)
            .paint(ctx, title_bounds);

        let button_bounds = Rect::new(...);
        Button::new("Create vault", button_bounds)
            .accent(true)
            .hit_id(HIT_CREATE)
            .paint(ctx, hits);
    }
}
```

The app constructs the root rect and passes it in:

```rust
let bounds = Rect::new(0.0, 0.0, logical_width, logical_height);
welcome.paint(&mut ctx, &mut hits, bounds);
```

## Layout helpers on Rect

`Rect` provides methods for common layout operations so components don't need manual coordinate math:

| Method | Purpose |
|---|---|
| `center_child(w, h)` | Returns a child rect centered inside this rect |
| `split_vertical(left_w)` | Splits into left/right pair at a pixel offset |
| `split_horizontal(top_h)` | Splits into top/bottom pair at a pixel offset |
| `inset(amount)` | Shrinks by `amount` on all sides |
| `contains(x, y)` | Hit testing — is a point inside? |
| `to_kurbo()` / `to_rounded(r)` | Convert to vello drawing primitives |

## Theme as single source of truth

All colors and typography tokens live in `Theme`. Components read from `ctx.theme` — no color constants in screen files.

```rust
pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub separator: Color,
    pub border: Color,
    pub accent: Color,
    pub accent_dim: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub typography: Typography,
}
```

## Hit testing

Components register clickable regions during `paint()` by pushing to `HitSink`. The app resolves clicks centrally after rendering:

```rust
// During paint:
hits.push(HIT_CREATE_VAULT, button_bounds);

// During mouse click (in app.rs):
if let Some(id) = hits.test(cursor_x, cursor_y) {
    let action = WelcomeAction::from_hit(id);
    handle(action);
}
```

Last-pushed wins on overlap, matching visual z-order (components painted later are visually on top).

## Testing

Since components paint within arbitrary bounds, tests can pass any rect size and verify behavior without coupling to window dimensions:

```rust
let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
screen.paint(&mut ctx, &mut hits, bounds);

// hit test at center of bounds
let (cx, cy) = bounds.center();
assert!(hits.test(cx, cy).is_some());
```

## Action system (command-palette-ready)

All editor operations are modeled as variants of a flat `Action` enum in `src/action.rs`. Keybindings resolve `(key, modifiers) -> Option<Action>` via `resolve_action()`, and `EditorView::handle_action()` dispatches the action to the active tab.

This design enables a future command palette to:
1. Enumerate `Action` variants to build a searchable list.
2. Display human-readable names for each action.
3. Execute through the same `handle_action` dispatch path — no separate code path for palette-triggered vs keybinding-triggered actions.

```
Key press -> resolve_action(key, modifiers) -> Action -> editor.handle_action(&action)
                                                  ^
                                                  |
                              Command palette -----+
```

Adding a new action requires: (1) add variant to `Action`, (2) add keybinding in `resolve_action`, (3) handle in `EditorView::handle_action`.

## What we intentionally skip

- **Element trait / layout engine** — not needed with fewer than 5 component types. If we reach 10+, consider a `trait Element { fn paint(&self, ctx, bounds); }`.
- **Button hover/focus states** — Button-specific hover can be added as a `hovered: bool` field later. File tree and tab bar hover is implemented via `cursor_position` in `DrawContext`.
- **Flexbox / auto-layout** — explicit Rect math is simpler and sufficient at this scale.
- **Animation, async, arena allocators** — irrelevant at this codebase size.
