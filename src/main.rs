mod app;
mod editor_view;
mod error;
mod file_tree;
mod global_config;
mod gpu;
mod text;
mod ui;
mod vault;
mod vault_config;
mod welcome;

use app::App;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    let mut app = App::new();
    if let Err(error) = event_loop.run_app(&mut app) {
        log::error!("Event loop error: {error}");
    }
}
