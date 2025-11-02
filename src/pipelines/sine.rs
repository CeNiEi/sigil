use std::{num::NonZero, slice::Iter};

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites,
    Device, Face, FragmentState, FrontFace, IndexFormat, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    boundary::Boundary,
    global::Global,
    ui::UiSineWaveData,
    utils::{BindGroupData, InstanceBufferData, VertexBufferData},
    vertex::Vertex,
};

pub(crate) struct Sine {
    pub(crate) boundary: Boundary,
    pub(crate) wave_data: Waves,
}

#[derive(Clone, Debug)]
pub(crate) struct Waves(pub(crate) Vec<SineWaveData>);

impl Waves {
    fn create_instance_buffer_data(&self, device: &Device) -> InstanceBufferData {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Wave Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.0),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        const F32X2_SIZE: u64 = std::mem::size_of::<[f32; 2]>() as u64;

        const F32_SIZE: u64 = std::mem::size_of::<f32>() as u64;

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<SineWaveData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    shader_location: 1,
                    offset: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    shader_location: 2,
                    offset: F32X2_SIZE,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    shader_location: 3,
                    offset: F32X2_SIZE + F32_SIZE,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    shader_location: 4,
                    offset: F32X2_SIZE + 2 * F32_SIZE,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    shader_location: 5,
                    offset: F32X2_SIZE + 3 * F32_SIZE,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    shader_location: 6,
                    offset: F32X2_SIZE + 4 * F32_SIZE,
                },
            ],
        };

        InstanceBufferData {
            vertex_buffer,
            vertex_buffer_layout,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub(crate) struct SineWaveData {
    pub(crate) center: [f32; 2],

    pub(crate) inner_radius: f32,
    pub(crate) thickness: f32,

    pub(crate) amplitude: f32,
    pub(crate) cycles: f32,

    pub(crate) speed: f32,
    _padding: f32,
}

impl Default for SineWaveData {
    fn default() -> Self {
        Self {
            amplitude: 0.05,
            center: [0.5, 0.5],
            inner_radius: 0.50,
            thickness: 0.01,
            cycles: 8.,
            speed: 0.005,
            _padding: 0.,
        }
    }
}

impl SineWaveData {
    pub(crate) fn new(
        center: [f32; 2],

        inner_radius: f32,
        thickness: f32,

        amplitude: f32,
        cycles: f32,

        speed: f32,
    ) -> Self {
        Self {
            speed,
            cycles,
            center,
            inner_radius,
            thickness,
            amplitude,
            _padding: 0.,
        }
    }

    fn create_bind_group_data(&self, device: &Device) -> BindGroupData {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(self),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: NonZero::new(std::mem::size_of::<Self>() as u64),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        BindGroupData {
            layout,
            buffer,
            bind_group,
        }
    }
}

pub(crate) struct SinePipeline {
    boundary_buffer_data: VertexBufferData,
    sinewave_instance_buffer_data: InstanceBufferData,
    global_bind_group_data: BindGroupData,
    global: Global,
    sine: Sine,
    pipeline: RenderPipeline,
}

impl SinePipeline {
    pub(crate) fn new(
        sine: Sine,
        global: Global,
        texture_format: TextureFormat,
        device: &Device,
    ) -> Self {
        let shader_module = device.create_shader_module(include_wgsl!("sine.wgsl"));

        let global_bind_group_data = global.create_bind_group_data(device);
        let sinewave_instance_buffer_data = sine.wave_data.create_instance_buffer_data(device);

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Sine Pipeline Layout"),
            bind_group_layouts: &[&global_bind_group_data.layout],
            ..Default::default()
        });

        let boundary_buffer_data = sine.boundary.create_vertex_buffer_data(device);

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Sine Pipeline"),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[
                    boundary_buffer_data.vertex_buffer_layout.clone(),
                    sinewave_instance_buffer_data.vertex_buffer_layout.clone(),
                ],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multiview: None,
            cache: None,
            multisample: MultisampleState::default(),
            layout: Some(&layout),
        });

        Self {
            global,
            sine,
            boundary_buffer_data,
            global_bind_group_data,
            sinewave_instance_buffer_data,
            pipeline,
        }
    }

    pub(crate) fn set_render_pass(&self, render_pass: &mut RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.boundary_buffer_data.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(
            1,
            self.sinewave_instance_buffer_data.vertex_buffer.slice(..),
        );
        render_pass.set_index_buffer(
            self.boundary_buffer_data.index_buffer.slice(..),
            IndexFormat::Uint16,
        );
        render_pass.set_bind_group(0, &self.global_bind_group_data.bind_group, &[]);
        render_pass.draw_indexed(0..6, 0, 0..self.sine.wave_data.0.len() as u32);
    }

    pub(crate) fn update_global_frame(&mut self, queue: &Queue) {
        self.global.increment_frame();
        queue.write_buffer(
            &self.global_bind_group_data.buffer,
            0,
            bytemuck::bytes_of(&self.global),
        );
    }

    pub(crate) fn update_sine_wave_data<'a>(
        &'a mut self,
        sine_wave_data: impl IntoIterator<Item = &'a UiSineWaveData>,
        queue: &Queue,
    ) {
        self.sine
            .wave_data
            .0
            .iter_mut()
            .zip(sine_wave_data)
            .for_each(|(old_data, new_data)| {
                old_data.center = new_data.center;
                old_data.amplitude = new_data.amplitude;
                old_data.inner_radius = new_data.inner_radius;
                old_data.thickness = new_data.thickness;
                old_data.cycles = new_data.cycles;
                old_data.speed = new_data.speed;
                old_data.center = new_data.center;
            });

        queue.write_buffer(
            &self.sinewave_instance_buffer_data.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.sine.wave_data.0),
        );
    }

    pub(crate) fn update_global_resolution(
        &mut self,
        new_width: u32,
        new_height: u32,

        queue: &Queue,
    ) {
        self.global.set_resolution(new_width, new_height);

        queue.write_buffer(
            &self.global_bind_group_data.buffer,
            0,
            bytemuck::bytes_of(&self.global),
        );
    }
}
