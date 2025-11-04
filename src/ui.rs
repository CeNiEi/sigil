use std::array;

use egui::{Context, ViewportId};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::State;
use wgpu::{
    Color, CommandEncoder, Device, LoadOp, Operations, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};
use winit::{event::WindowEvent, window::Window};

pub(crate) struct Ui {
    renderer: Renderer,
    state: State,
    pub(crate) waves: UiWaves,
}

pub(crate) struct UiWaves(pub(crate) [UiSineWaveData; 8]);

impl Default for UiWaves {
    fn default() -> Self {
        let mut waves = UiWaves(array::from_fn(|_| UiSineWaveData::default()));

        waves.0[0].init = true;

        waves
    }
}

pub(crate) struct UiSineWaveData {
    pub(crate) amplitude: f32,
    pub(crate) center: [f32; 2],
    pub(crate) inner_radius: f32,
    pub(crate) thickness: f32,
    pub(crate) cycles: f32,
    pub(crate) speed: f32,
    pub(crate) init: bool,
}

impl Default for UiSineWaveData {
    fn default() -> Self {
        Self {
            amplitude: 0.05,
            center: [0.5, 0.5],
            inner_radius: 0.50,
            thickness: 0.01,
            cycles: 8.,
            speed: 0.005,
            init: false,
        }
    }
}

impl Ui {
    pub(crate) fn new(device: &Device, format: TextureFormat, window: &Window) -> Self {
        let renderer = Renderer::new(device, format, RendererOptions::default());
        let context = Context::default();

        let state = State::new(context.clone(), ViewportId::ROOT, window, None, None, None);
        let waves = UiWaves::default();

        Self {
            renderer,
            state,
            waves,
        }
    }

    fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);
    }

    fn end_frame(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let full_output = self.state.egui_ctx().end_pass();

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        let size = window.inner_size();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,

                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            label: Some("UI Render Pass"),
            ..Default::default()
        });

        self.renderer.render(
            &mut render_pass.forget_lifetime(),
            &tris,
            &screen_descriptor,
        );

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }

    pub(crate) fn panel(&mut self) {
        egui::Window::new("Control Panel")
            .resizable(true)
            .vscroll(true)
            .default_open(false)
            .movable(true)
            .show(self.state.egui_ctx(), |ui| {
                ui.horizontal(|ui| {
                    if let Some(pos) = self.waves.0.iter().position(|wave_data| !wave_data.init) {
                        let response = ui.button("Add Wave");
                        if response.clicked() {
                            self.waves.0[pos].init = true;
                        }
                    };

                    ui.separator();
                });

                ui.separator();

                self.waves
                    .0
                    .iter_mut()
                    .filter(|wave_data| wave_data.init)
                    .for_each(|sine_wave_data| {
                        ui.horizontal(|ui| {
                            ui.label("Center: ");
                            ui.add(
                                egui::Slider::new(&mut sine_wave_data.center[0], 0.0..=1.0)
                                    .text("X"),
                            );
                            ui.add(
                                egui::Slider::new(&mut sine_wave_data.center[1], 0.0..=1.0)
                                    .text("Y"),
                            );
                        });
                        ui.separator();

                        ui.add(
                            egui::Slider::new(&mut sine_wave_data.amplitude, 0.0..=0.1)
                                .text("Amplitude"),
                        );
                        ui.separator();

                        ui.add(
                            egui::Slider::new(&mut sine_wave_data.inner_radius, 0.0..=1.)
                                .text("Inner Radius"),
                        );
                        ui.separator();

                        ui.add(
                            egui::Slider::new(&mut sine_wave_data.thickness, 0.01..=0.1)
                                .text("Thickness"),
                        );

                        ui.separator();

                        ui.add(
                            egui::Slider::new(&mut sine_wave_data.cycles, 1.0..=16.)
                                .step_by(1.)
                                .text("Cycles"),
                        );

                        ui.separator();

                        ui.add(
                            egui::Slider::new(&mut sine_wave_data.speed, -0.1..=0.1).text("Speed"),
                        );
                        ui.separator();
                        ui.separator();
                    });
            });
    }

    pub(crate) fn render(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        texture_view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        self.begin_frame(window);

        self.panel();

        self.end_frame(window, device, queue, texture_view, encoder);
    }

    pub(crate) fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }
}
