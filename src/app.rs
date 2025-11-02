use winit::{application::ApplicationHandler, event::WindowEvent, window::Window};

use crate::render::Render;

#[derive(Default)]
pub(crate) enum App {
    Initialized {
        render: Render,
    },
    #[default]
    Uninitialized,
}

impl App {
    fn is_initialized(&self) -> bool {
        !matches!(self, Self::Uninitialized)
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.is_initialized() {
            return;
        }

        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("Failed to create Window");

        let render = pollster::block_on(Render::new(window)).expect("Failed to create render");

        *self = Self::Initialized { render };
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Self::Initialized { render } = self else {
            return;
        };

        render.handle_ui_inputs(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                render.resize(physical_size);
            }

            WindowEvent::RedrawRequested => match render.render() {
                Ok(()) => {}
                Err(_) => {
                    event_loop.exit();
                }
            },
            _ => (),
        }
    }
}
