# Milestone 1 — Text on Screen: Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix the glyph blit stage so that a string of text is visibly rendered on screen.

**Architecture:** cosmic-text lays out and rasterizes glyphs via SwashCache; the resulting pixel images must be composited into the Vello scene before GPU submission. Two glyph image types exist: `Content::Mask` (alpha mask + foreground colour) and `Content::Color` (RGBA bitmap). The fix touches only `src/render/mod.rs` and `src/app.rs`.

**Tech Stack:** Rust, wgpu 27, vello 0.7, cosmic-text 0.12 (swash transitive dep)

---

## Background

The three-stage glyph pipeline is:

1. **Layout** — cosmic-text `TextBuffer::shape_until_scroll` → per-glyph physical positions + cache keys ✅
2. **Rasterize** — `SwashCache::get_image` → pixel buffer in CPU memory ✅ (cache warmed)
3. **Blit** — composite pixel buffer into Vello scene ❌ **missing**

All three draw functions (`draw_buffer`, `draw_render_lines`, `draw_render_lines_offset`) warm the swash cache but discard the image instead of drawing it. Stage 3 is the only blocker.

### How Vello accepts raster images

Vello's `Scene` accepts raster images via `Scene::draw_image`:

```rust
pub fn draw_image(&mut self, image: &vello::peniko::Image, transform: Affine)
```

`peniko::Image` wraps a `peniko::Blob<u8>` (raw RGBA bytes) plus width/height. For mask glyphs, convert the single-channel alpha mask to RGBA by applying the foreground colour per pixel. For colour glyphs, pass data through directly.

---

## Task 1 — Write failing tests for the swash→rgba conversion helper

**Files:**
- Modify: `src/render/mod.rs` (add `#[cfg(test)]` module)

**Step 1: Add the test module at the bottom of `src/render/mod.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use vello::peniko::Color;

    #[test]
    fn mask_glyph_expands_to_rgba() {
        let image = cosmic_text::SwashImage {
            placement: cosmic_text::Placement { left: 0, top: 0, width: 2, height: 1 },
            content: cosmic_text::SwashContent::Mask,
            data: vec![128u8, 255u8],
        };
        let fg = Color::from_rgba8(255, 200, 0, 255);
        let rgba = swash_to_rgba(&image, fg);
        assert_eq!(rgba.len(), 8);
        assert_eq!(&rgba[0..4], &[255, 200, 0, 128]);
        assert_eq!(&rgba[4..8], &[255, 200, 0, 255]);
    }

    #[test]
    fn color_glyph_passes_through() {
        let image = cosmic_text::SwashImage {
            placement: cosmic_text::Placement { left: 0, top: 0, width: 1, height: 1 },
            content: cosmic_text::SwashContent::Color,
            data: vec![10u8, 20u8, 30u8, 200u8],
        };
        let fg = Color::from_rgba8(0, 0, 0, 255);
        let rgba = swash_to_rgba(&image, fg);
        assert_eq!(rgba, vec![10, 20, 30, 200]);
    }
}
```

**Step 2: Run to confirm they fail**

```bash
cargo test swash_to_rgba 2>&1 | head -20
```

Expected: compile error — `swash_to_rgba` not defined yet.

**Step 3: Commit**

```bash
git add src/render/mod.rs
git commit -m "test(render): add failing tests for swash→rgba conversion"
```

---

## Task 2 — Implement swash_to_rgba and make tests pass

**Files:**
- Modify: `src/render/mod.rs`

**Step 1: Add the conversion helper above the test module**

```rust
/// Converts a swash glyph image to a flat RGBA byte vec for peniko::Image.
/// Mask glyphs are tinted with `fg`; colour glyphs pass data through unchanged.
fn swash_to_rgba(image: &cosmic_text::SwashImage, fg: vello::peniko::Color) -> Vec<u8> {
    match image.content {
        cosmic_text::SwashContent::Mask | cosmic_text::SwashContent::SubpixelMask => {
            image.data.iter().flat_map(|&alpha| [fg.r, fg.g, fg.b, alpha]).collect()
        }
        cosmic_text::SwashContent::Color => image.data.to_vec(),
    }
}
```

**Step 2: Run tests**

```bash
cargo test 2>&1 | tail -20
```

Expected: both tests pass.

**Step 3: Commit**

```bash
git add src/render/mod.rs
git commit -m "feat(render): implement swash→rgba conversion helper"
```

---

## Task 3 — Add peniko Image import and blit helper

**Files:**
- Modify: `src/render/mod.rs`

**Step 1: Extend the vello import at line 7**

Change:

```rust
use vello::peniko::{Brush, Color, Fill};
```

To:

```rust
use vello::peniko::{Blob, Brush, Color, Fill, Format, Image as PenikoImage};
```

**Step 2: Add a private blit helper that draws one glyph into self.scene**

Add this method inside `impl Renderer`:

```rust
/// Blits a single rasterized glyph into the scene at its physical screen position.
fn blit_glyph(
    &mut self,
    physical: &cosmic_text::PhysicalGlyph,
    fg: Color,
) {
    let Some(swash_image) = self.swash_cache.get_image(&mut self.font_system, physical.cache_key) else {
        return;
    };
    let width = swash_image.placement.width;
    let height = swash_image.placement.height;
    if width == 0 || height == 0 {
        return;
    }
    let rgba = swash_to_rgba(swash_image, fg);
    let blob = Blob::new(std::sync::Arc::new(rgba));
    let image = PenikoImage::new(blob, Format::Rgba8, width, height);
    let glyph_x = (physical.x + swash_image.placement.left) as f64;
    let glyph_y = (physical.y - swash_image.placement.top) as f64;
    self.scene.draw_image(&image, Affine::translate((glyph_x, glyph_y)));
}
```

**Step 3: Build**

```bash
cargo build 2>&1 | tail -30
```

Fix any type mismatches. Common issues:
- `placement.left` / `placement.top` are `i32` — the cast `as f64` handles this.
- `Format::Rgba8` variant name may differ — check with `cargo doc --package peniko`.
- Borrow conflict: `self.swash_cache` borrows `self.font_system` mutably; `self.scene` is a separate field so the borrow checker allows it in Rust 2021 if they are distinct struct fields.

**Step 4: Commit**

```bash
git add src/render/mod.rs
git commit -m "feat(render): add blit_glyph helper to Renderer"
```

---

## Task 4 — Wire blit_glyph into all three draw functions

**Files:**
- Modify: `src/render/mod.rs:112–118`, `src/render/mod.rs:187–194`, `src/render/mod.rs:265–271`

**Step 1: Replace the TODO block in draw_buffer (lines 112–118)**

Old:

```rust
for run in text_buf.layout_runs() {
    for glyph in run.glyphs.iter() {
        let physical = glyph.physical((left_pad, y), 1.0);
        // Rasterise via swash so the glyph cache is warm; full blit in Milestone 3.
        let _ = self.swash_cache.get_image(&mut self.font_system, physical.cache_key);
    }
}
```

New:

```rust
let fg = Color::from_rgba8(220, 220, 220, 255);
for run in text_buf.layout_runs() {
    for glyph in run.glyphs.iter() {
        let physical = glyph.physical((left_pad, y), 1.0);
        self.blit_glyph(&physical, fg);
    }
}
```

**Step 2: Replace the TODO block in draw_render_lines (lines 187–194)**

Old:

```rust
for run in text_buf.layout_runs() {
    for glyph in run.glyphs.iter() {
        let physical = glyph.physical((x, y), 1.0);
        // Rasterise via swash to warm the glyph cache; full blit in Milestone 5.
        let _ = self.swash_cache.get_image(&mut self.font_system, physical.cache_key);
        x += glyph.w;
    }
}
```

New:

```rust
let fg = span_fg_color(&span.style);
for run in text_buf.layout_runs() {
    for glyph in run.glyphs.iter() {
        let physical = glyph.physical((x, y), 1.0);
        self.blit_glyph(&physical, fg);
        x += glyph.w;
    }
}
```

**Step 3: Apply the same replacement in draw_render_lines_offset (lines 265–271)**

Identical substitution to Step 2.

**Step 4: Add span_fg_color helper**

```rust
fn span_fg_color(style: &SpanStyle) -> Color {
    match style {
        SpanStyle::Code | SpanStyle::CodeBlockText => Color::from_rgba8(171, 200, 148, 255),
        SpanStyle::BulletMarker => Color::from_rgba8(97, 175, 239, 255),
        _ => Color::from_rgba8(220, 220, 220, 255),
    }
}
```

**Step 5: Build and run**

```bash
cargo build 2>&1 | tail -20
make run
```

Expected: window opens, text is visible on screen.

**Step 6: Commit**

```bash
git add src/render/mod.rs
git commit -m "feat(render): wire glyph blit into all draw functions — text on screen"
```

---

## Task 5 — Apply HiDPI scale factor

**Files:**
- Modify: `src/render/mod.rs` (add `scale_factor` parameter to draw functions)
- Modify: `src/app.rs` (pass `window.scale_factor()` to draw calls)

**Step 1: Read src/app.rs to find current draw call sites**

Search for `draw_render_lines` in `src/app.rs` to locate all call sites.

**Step 2: Add scale_factor parameter to draw_render_lines and draw_render_lines_offset**

Change signatures to accept `scale_factor: f32` and pass it to `glyph.physical((x, y), scale_factor)`.

For `draw_buffer`, same change.

**Step 3: Update call sites in app.rs**

At each call site pass `window.scale_factor() as f32`. Store `scale_factor` on `App` and update it in the `ScaleFactorChanged` handler if one exists, otherwise read from `window` directly.

**Step 4: Build and run on Retina display**

```bash
cargo build 2>&1 | tail -20
make run
```

Expected: characters are crisp (not blurry) on a Retina display.

**Step 5: Commit**

```bash
git add src/render/mod.rs src/app.rs
git commit -m "feat(render): apply HiDPI scale factor to glyph physical positions"
```

---

## Task 6 — Verify all acceptance criteria

**Step 1: Run full test suite**

```bash
cargo test 2>&1
```

Expected: all tests pass, no regressions.

**Step 2: Run the app and manually verify**

```bash
make run
```

Checklist:
- [ ] Window opens without panic
- [ ] Text is visibly rendered on screen
- [ ] Characters are crisp on Retina (no blur)
- [ ] Cursor rectangle is still rendered correctly

**Step 3: Final commit if anything was adjusted**

```bash
git add -p
git commit -m "fix(render): milestone 1 acceptance criteria — text on screen"
```

---

## Known Risks

| Risk | Mitigation |
|------|-----------|
| `peniko::Image::new` or `Format` path differs in vello 0.7 | Run `cargo doc --package peniko --open` to confirm API |
| `SwashImage` field names differ from docs | Run `cargo check` — error messages show actual field names |
| Glyph Y-offset sign wrong (text too high or low) | Toggle `- swash_image.placement.top` to `+ swash_image.placement.top` |
| Borrow conflict on `self` in blit_glyph | Extract `swash_image` data before the scene call; all fields are separate so Rust 2021 NLL handles it |
| scale_factor not stored on App | Read `window.scale_factor()` directly at each call site as a safe fallback |
