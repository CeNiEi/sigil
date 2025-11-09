use std::{array, sync::Arc};

use anyhow::{Result, anyhow};
use wgpu::{
    Backends, Color, Device, DeviceDescriptor, Extent3d, Instance, InstanceDescriptor, LoadOp,
    Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions,
    StoreOp, Surface, SurfaceConfiguration, Texture, TextureDimension, TextureFormat,
    TextureUsages, TextureView,
    wgt::{CommandEncoderDescriptor, TextureDescriptor, TextureViewDescriptor},
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{
    boundary::Boundary,
    global::Global,
    pipelines::{
        post::PostPipeline,
        sine::{Sine, SinePipeline, SineWaveData, Waves},
    },
    ui::Ui,
    vertex::Vertex,
};

pub(crate) struct Render {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    window: Arc<Window>,
    config: SurfaceConfiguration,
    sine_pipeline: SinePipeline,
    post_pipeline: PostPipeline,
    off_screen_texture: Texture,
    off_screen_texture_view: TextureView,
    ui: Ui,
}

impl Render {
    pub(crate) async fn new(window: Window) -> Result<Self> {
        let window = Arc::new(window);
        let window_size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::default(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device Descriptor"),
                ..Default::default()
            })
            .await?;

        let surface_compatibilities = surface.get_capabilities(&adapter);

        let surface_format = surface_compatibilities
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .or(surface_compatibilities.formats.first())
            .copied()
            .ok_or_else(|| anyhow!("Surface is incompatible with the adapter"))?;

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_compatibilities
                .present_modes
                .first()
                .copied()
                .ok_or_else(|| anyhow!("Surface is incompatible with the adapter"))?,
            alpha_mode: surface_compatibilities
                .alpha_modes
                .first()
                .copied()
                .ok_or_else(|| anyhow!("No supported alpha modes found"))?,
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let (off_screen_texture, off_screen_texture_view) =
            Self::create_off_screen_texture(config.width, config.height, config.format, &device);

        let ui = Ui::new(&device, config.format, &window);

        let sine = Sine {
            boundary: Boundary::new(
                Vertex::new(-1., 1.),
                Vertex::new(-1., -1.),
                Vertex::new(1., -1.),
                Vertex::new(1., 1.),
            ),
            wave_data: Waves::default(),
        };

        let global = Global::new(800, 600);

        let sine_pipeline = SinePipeline::new(sine, global, config.format, &device);

        let post_pipeline =
            PostPipeline::new(&off_screen_texture_view, config.format, global, &device);

        Ok(Self {
            ui,
            off_screen_texture,
            surface,
            device,
            sine_pipeline,
            queue,
            window,
            config,
            off_screen_texture_view,
            post_pipeline,
        })
    }

    fn create_off_screen_texture(
        width: u32,
        height: u32,
        format: TextureFormat,
        device: &Device,
    ) -> (Texture, TextureView) {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Off Screen Texture"),
            size: Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor {
            label: Some("Off Streen Texture View"),
            ..Default::default()
        });

        (texture, view)
    }

    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.sine_pipeline.update_global_resolution(
                new_size.width,
                new_size.height,
                &self.queue,
            );
            self.post_pipeline.update_global_resolution(
                new_size.width,
                new_size.height,
                &self.queue,
            );

            let (off_screen_texture, off_screen_texture_view) = Self::create_off_screen_texture(
                new_size.width,
                new_size.height,
                self.config.format,
                &self.device,
            );

            self.off_screen_texture = off_screen_texture;
            self.off_screen_texture_view = off_screen_texture_view;

            self.post_pipeline
                .update_off_screen_bindgroup(&self.off_screen_texture_view, &self.device);
        }
    }

    pub(crate) fn handle_ui_inputs(&mut self, event: &WindowEvent) {
        self.ui.handle_input(&self.window, event);
    }

    pub(crate) fn render(&mut self) -> Result<()> {
        self.window.request_redraw();

        self.sine_pipeline.update_global_frame(&self.queue);
        self.sine_pipeline
            .update_sine_wave_data(&self.ui.waves.0, &self.queue);

        let surface_texture = self.surface.get_current_texture()?;

        let texture_view = surface_texture.texture.create_view(&TextureViewDescriptor {
            label: Some("Texture View Descriptor"),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.off_screen_texture_view,

                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    resolve_target: None,
                    depth_slice: None,
                })],
                label: Some("Render Pass"),
                ..Default::default()
            });

            self.sine_pipeline.set_render_pass(&mut render_pass);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture_view,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    resolve_target: None,
                    depth_slice: None,
                })],
                label: Some("Post Render Pass"),
                ..Default::default()
            });

            self.post_pipeline.set_render_pass(&mut render_pass);
        }

        self.ui.render(
            &self.window,
            &self.device,
            &self.queue,
            &texture_view,
            &mut encoder,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
