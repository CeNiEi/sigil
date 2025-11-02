use anyhow::Result;
use winit::event_loop::EventLoop;

use crate::app::App;

mod app;
mod boundary;
mod global;
mod pipelines;
mod render;
mod ui;
mod utils;
mod vertex;

fn main() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = App::default();

    event_loop.run_app(&mut app)?;

    Ok(())
}
