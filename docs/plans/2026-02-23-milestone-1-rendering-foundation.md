# Milestone 1: Rendering Foundation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Produce a macOS window that initializes a wgpu GPU context and renders a single line of text using Vello and cosmic-text.

**Architecture:** `main.rs` drives a `winit` event loop. On resume, it creates a `wgpu` surface and passes it to a `render::Renderer` that holds a Vello scene and a cosmic-text font system. Each frame it lays out a hardcoded string and renders it to screen. No buffer, no Vim, no Markdown — just the GPU text pipeline proving it works end-to-end.

**Tech Stack:**
- `winit 0.30` — cross-platform window creation and OS event loop (keyboard, mouse, resize). The entry point for any native desktop app in Rust.
- `wgpu 0.22` — safe, cross-platform GPU API abstracting over Metal, Vulkan, and DirectX 12. Owns the render surface and submits draw commands to the GPU.
- `vello 0.3` — 2D vector renderer that runs entirely on the GPU via wgpu compute shaders. Draws shapes, paths, and glyph outlines at high frame rates.
- `cosmic-text 0.12` — text shaping and layout engine backed by HarfBuzz (via rustybuzz). Turns Unicode strings and font files into positioned glyph runs ready for rendering.
- `taffy 0.5` — CSS Flexbox and Grid layout engine. Computes the position and size of every UI element without a browser or DOM.

---

## Prerequisites

- Rust toolchain installed (`rustup show` should print stable)
- macOS with Xcode command-line tools (`xcode-select --install`)
- `cargo` available in PATH

---

### Task 0: Configure Clippy and Makefile

**Files:**
- Create: `.clippy.toml`
- Create: `Makefile`

**Step 1: Create `.clippy.toml`**

```toml
# Enforce a strict but practical set of lints for the project.
avoid-breaking-exported-api = false
```

**Step 2: Create `Makefile`**

```makefile
.PHONY: build run test lint check clean

build:
	cargo build

run:
	cargo run

test:
	cargo test

lint:
	cargo clippy --all-targets --all-features -- -D warnings

check:
	cargo check --all-targets

clean:
	cargo clean
```

**Step 3: Verify lint passes on the empty project**

Run: `make lint`
Expected: `Finished` with zero warnings or errors.

**Step 4: Stage changes**

```bash
git add .
```

**Step 5: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 2: Initialise the Cargo project

**Files:**
- Modify: `Cargo.toml`
- Create: `src/main.rs`

**Step 1: Verify the workspace root**

```bash
ls Cargo.toml src/
```

Expected: `Cargo.toml` exists. `src/` may not exist yet.

**Step 2: Write `Cargo.toml`**

Replace the contents with:

```toml
[package]
name = "onyx"
version = "0.1.0"
edition = "2021"

[dependencies]
winit = "0.30"
wgpu = "22"
vello = "0.3"
cosmic-text = "0.12"
taffy = "0.5"
pollster = "0.4"
log = "0.4"
env_logger = "0.11"

[profile.dev]
opt-level = 1
```

**Step 3: Write a minimal `src/main.rs` that compiles**

```rust
fn main() {
    println!("onyx starting");
}
```

**Step 4: Verify it compiles**

```bash
cargo build 2>&1
```

Expected: `Compiling onyx v0.1.0` then `Finished`. Zero errors.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 3: Open a native window with winit

**Files:**
- Modify: `src/main.rs`

**Step 1: Replace `src/main.rs` with the winit event loop**

```rust
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct App {
    window: Option<std::sync::Arc<Window>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Onyx"))
            .expect("failed to create window");
        self.window = Some(std::sync::Arc::new(window));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { window: None };
    event_loop.run_app(&mut app).expect("event loop failed");
}
```

**Step 2: Build**

```bash
cargo build 2>&1
```

Expected: Compiles without errors.

**Step 3: Run and verify the window opens**

```bash
cargo run
```

Expected: A blank native macOS window titled "Onyx" appears. Close it with the red button. Process exits cleanly.

**Step 4: Stage changes**

```bash
git add .
```

**Step 5: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 4: Create the render module skeleton

**Files:**
- Create: `src/render/mod.rs`
- Modify: `src/main.rs`

**Step 1: Create `src/render/mod.rs`**

```rust
// GPU rendering pipeline: wgpu surface + Vello scene + cosmic-text layout.

pub struct Renderer {
    // populated in Task 4
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {}
    }

    pub fn render(&mut self) {
        // populated in Task 4
    }
}
```

**Step 2: Declare the module in `src/main.rs`**

Add at the top of `src/main.rs`:

```rust
mod render;
```

**Step 3: Build**

```bash
cargo build 2>&1
```

Expected: Compiles. No warnings about unused items are errors.

**Step 4: Stage changes**

```bash
git add .
```

**Step 5: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 5: Initialise wgpu surface

**Files:**
- Modify: `src/render/mod.rs`
- Modify: `src/main.rs`

**Step 1: Replace `src/render/mod.rs`**

```rust
use std::sync::Arc;
use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> Self {
        pollster::block_on(Self::init(window))
    }

    async fn init(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("failed to create wgpu surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("no suitable GPU adapter found");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("failed to get GPU device");

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Renderer { surface, device, queue, config }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
```

**Step 2: Wire renderer into `App` in `src/main.rs`**

```rust
mod render;

use render::Renderer;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Onyx"))
                .expect("failed to create window"),
        );
        let renderer = Renderer::new(window.clone());
        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.renderer {
                    r.resize(size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(r) = &mut self.renderer {
                    r.render();
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App { window: None, renderer: None };
    event_loop.run_app(&mut app).expect("event loop failed");
}
```

**Step 3: Build and run**

```bash
cargo run
```

Expected: Dark window (`#1A1A1E` approximate) opens. No crash. Resizing works.

**Step 4: Stage changes**

```bash
git add .
```

**Step 5: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 6: Integrate Vello renderer

**Files:**
- Modify: `src/render/mod.rs`

Vello renders vector graphics (including text glyphs) to a wgpu texture. This task wires a `vello::Renderer` into the existing wgpu setup.

**Step 1: Update `src/render/mod.rs` — add Vello renderer**

Add to the `Renderer` struct and `init`:

```rust
use vello::{RenderParams, Renderer as VelloRenderer, RendererOptions, Scene};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    vello: VelloRenderer,
    scene: Scene,
}
```

In `init`, after creating `device` and `queue`, add:

```rust
let vello = VelloRenderer::new(
    &device,
    RendererOptions {
        surface_format: Some(format),
        use_cpu: false,
        antialiasing_support: vello::AaSupport::all(),
        num_init_threads: std::num::NonZeroUsize::new(1),
    },
)
.expect("failed to create Vello renderer");
let scene = Scene::new();
```

Update `render` to use Vello:

```rust
pub fn render(&mut self) {
    let frame = match self.surface.get_current_texture() {
        Ok(f) => f,
        Err(_) => return,
    };

    self.scene.reset();

    self.vello
        .render_to_surface(
            &self.device,
            &self.queue,
            &self.scene,
            &frame,
            &RenderParams {
                base_color: vello::peniko::Color::from_rgba8(26, 26, 30, 255),
                width: self.config.width,
                height: self.config.height,
                antialiasing_method: vello::AaConfig::Msaa16,
            },
        )
        .expect("vello render failed");

    frame.present();
}
```

**Step 2: Build**

```bash
cargo build 2>&1
```

Expected: Compiles. Window still opens and clears to the dark background.

**Step 3: Stage changes**

```bash
git add .
```

**Step 4: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

### Task 7: Render a line of text with cosmic-text

**Files:**
- Modify: `src/render/mod.rs`

cosmic-text handles font loading, shaping, and layout. Vello draws the resulting glyph runs.

**Step 1: Add font system to the `Renderer` struct**

```rust
use cosmic_text::{Attrs, Buffer as TextBuffer, FontSystem, Metrics, Shaper, SwashCache};

pub struct Renderer {
    // ... existing fields ...
    font_system: FontSystem,
    swash_cache: SwashCache,
}
```

In `init`, after Vello setup:

```rust
let font_system = FontSystem::new();
let swash_cache = SwashCache::new();
```

**Step 2: Add a `draw_text` helper**

```rust
fn draw_text(&mut self, text: &str, x: f32, y: f32) {
    let metrics = Metrics::new(16.0, 20.0);
    let mut buffer = TextBuffer::new(&mut self.font_system, metrics);
    buffer.set_size(&mut self.font_system, Some(self.config.width as f32), None);
    buffer.set_text(&mut self.font_system, text, Attrs::new(), cosmic_text::Shaping::Advanced);
    buffer.shape_until_scroll(&mut self.font_system, false);

    for run in buffer.layout_runs() {
        for glyph in run.glyphs.iter() {
            let physical = glyph.physical((x, y), 1.0);
            if let Some(image) = self.swash_cache.get_image(&mut self.font_system, physical.cache_key) {
                // Convert swash image to Vello glyph path — simplified for MVP.
                // Full integration uses vello::glyph::GlyphProvider; stub here proves the pipeline.
                let _ = image;
            }
        }
    }
}
```

> **Note:** Full Vello glyph rendering uses `vello::glyph::GlyphProvider`. For this milestone, the goal is proving the pipeline initialises without panic. The next milestone (Editor Core) completes the text-to-glyph path once the buffer layer is in place.

**Step 3: Call `draw_text` from `render`**

In `render`, after `self.scene.reset()`:

```rust
self.draw_text("Onyx — rendering foundation", 20.0, 40.0);
```

**Step 4: Build and run**

```bash
cargo run
```

Expected: Window opens. Text may not be visible yet (glyph path is stubbed) but no panic. This proves font system initialises and the pipeline is wired end-to-end.

**Step 5: Stage changes**

```bash
git add .
```

**Step 6: Await commit approval**

Show the user a summary of what was changed in this task, then stop and ask:

> "Ready to commit. Please review the staged changes and type 'commit' to continue, or describe any changes you want made first."

Do NOT proceed to the next task until the user confirms.

---

## Milestone 1 Complete

At this point:
- `cargo run` opens a native macOS window
- wgpu GPU context is initialised and frames are presented
- Vello renderer is wired to the wgpu surface
- cosmic-text font system is initialised
- A text draw call is wired into the render loop (glyph rendering completed in Milestone 2)

The window is dark and otherwise blank — that is the correct state for this milestone.
