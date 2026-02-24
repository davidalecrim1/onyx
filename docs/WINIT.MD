# Winit

Cross-platform window creation and OS event loop for Rust. Renderer-agnostic — exposes raw window/display handles for wgpu, Vulkan, Metal, OpenGL.

## Core Types

| Type | Role |
|------|------|
| `EventLoop<T>` | Root event pump. Created once, consumed by `run_app()`. Main thread only. `T` = user event type (default `()`) |
| `ActiveEventLoop` | Borrowed ref in every handler method. Creates windows, calls `exit()` |
| `Window` | One OS window. Wrap in `Arc` to share with renderer |
| `WindowId` | Opaque ID routing events when multiple windows exist |
| `WindowAttributes` | Window config builder (replaces removed `WindowBuilder` in 0.30) |
| `EventLoopProxy<T>` | `Send` handle for cross-thread `user_event()` dispatch |
| `ControlFlow` | `Poll` (spin), `Wait` (block), `WaitUntil(Instant)` (block with timeout) |

## ApplicationHandler Trait

Replaces the old closure-based `EventLoop::run()` in 0.30+.

| Method | When |
|--------|------|
| `resumed()` | App ready — create windows here (required on iOS/macOS) |
| `window_event()` | Per-window OS event |
| `suspended()` | App backgrounded — drop GPU surfaces |
| `about_to_wait()` | Event queue drained |
| `new_events()` | Before a batch of events |
| `user_event()` | Custom event via `EventLoopProxy` |
| `device_event()` | Raw unfiltered hardware input |
| `exiting()` | Loop shutting down (irreversible) |

## Event Flow

```
OS → EventLoop::run_app()
       → new_events()
       → window_event() / device_event() / user_event()  (per event)
       → about_to_wait()
       → (sleep or poll depending on ControlFlow)
```

## Key WindowEvent Variants

| Variant | Description |
|---------|-------------|
| `RedrawRequested` | Render this frame |
| `Resized(PhysicalSize)` | Surface must be rebuilt |
| `ScaleFactorChanged` | DPI changed; recalculate layout |
| `CloseRequested` | User pressed close |
| `KeyboardInput` | `KeyEvent` with `PhysicalKey` + logical `Key` |
| `ModifiersChanged` | Ctrl/Shift/Alt/Meta state |
| `MouseInput` | Button press/release |
| `CursorMoved` | Pointer position (physical pixels) |
| `MouseWheel` | Scroll delta |

## Keyboard Input (0.30+)

Uses `KeyEvent` with `PhysicalKey` (scancode) and `Key` (logical):

```rust
WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state, .. }, .. } => {
    if state == ElementState::Pressed {
        match logical_key {
            Key::Character(c) => { /* text */ }
            Key::Named(NamedKey::Escape) => { /* special */ }
            _ => {}
        }
    }
}
```

## Sizing Model

- `PhysicalSize<u32>` — pixel-accurate, pass to GPU surface
- `LogicalSize<f64>` — DPI-independent, use for UI layout
- Convert: `physical = logical * scale_factor`

## Minimal Skeleton

```rust
// main.rs
let event_loop = EventLoop::new()?;
event_loop.set_control_flow(ControlFlow::Poll);
event_loop.run_app(&mut App::new())?;

// app.rs
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes().with_title("App")
            ).unwrap()
        );
        self.renderer = Some(Renderer::new(window.clone()));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.renderer.resize(size),
            WindowEvent::RedrawRequested => {
                self.renderer.render();
                self.window.request_redraw(); // continuous loop
            }
            _ => {}
        }
    }
}
```

## Cross-Thread Events

```rust
let proxy = event_loop.create_proxy();
// any thread:
proxy.send_event(MyEvent::Something)?;
// handler:
fn user_event(&mut self, _: &ActiveEventLoop, event: MyEvent) { ... }
```

## Platform Backends

Windows (Win32), macOS (AppKit), Linux (X11/Wayland), iOS (UIKit), Android (NDK), Web (Canvas).
