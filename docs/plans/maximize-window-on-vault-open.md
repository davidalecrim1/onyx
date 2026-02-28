# Plan: Maximize window when opening a vault

## Context

The app currently starts with a fixed 900x600 window (welcome screen). When the user opens or creates a vault, the app transitions to the editor view but the window stays at 900x600. Like VS Code and Zed, the window should maximize when a vault is opened.

## Approach

Call `window.set_maximized(true)` right after transitioning to `AppScreen::Editor` in `handle_vault_action()`.

## Changes

### `src/app.rs`

1. After setting `self.screen = AppScreen::Editor(...)`, call:
   ```rust
   if let Some(ref window) = self.window {
       window.set_maximized(true);
   }
   ```
2. Do this in both the `CreateVault` and `OpenVault` arms (lines 59 and 73).

No other files need changes. `winit::window::Window::set_maximized(true)` is the standard winit API — it maximizes without going fullscreen, exactly like VS Code/Zed behavior.

## Verification

1. `make format && make lint` — no warnings
2. Run the app, confirm the welcome screen is still 900x600
3. Open a vault — window should maximize to fill the screen (not fullscreen)
4. Create a vault — same maximize behavior
