use std::num::NonZero;

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BlendState, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites,
    Device, Face, FragmentState, FrontFace, IndexFormat, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureFormat, VertexState, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    boundary::Boundary,
    global::Global,
    utils::{BindGroupData, BufferData},
};

pub(crate) struct Sine {
    pub(crate) boundary: Boundary,
    pub(crate) wave_data: SineWaveData,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub(crate) struct SineWaveData {
    pub(crate) center: [f32; 2],

    pub(crate) inner_radius: f32,
    pub(crate) outer_radius: f32,

    pub(crate) amplitude: f32,
    pub(crate) cycles: f32,

    pub(crate) speed: f32,
    _padding: f32,
}

impl SineWaveData {
    pub(crate) fn new(
        center: [f32; 2],

        inner_radius: f32,
        outer_radius: f32,

        amplitude: f32,
        cycles: f32,

        speed: f32,
    ) -> Self {
        Self {
            speed,
            cycles,
            center,
            inner_radius,
            outer_radius,
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
    pub(crate) boundary_buffer_data: BufferData,
    pub(crate) sinewave_bind_group_data: BindGroupData,
    pub(crate) global_bind_group_data: BindGroupData,
    pub(crate) global: Global,
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
        let sinewave_bind_group_data = sine.wave_data.create_bind_group_data(device);

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Sine Pipeline Layout"),
            bind_group_layouts: &[
                &global_bind_group_data.layout,
                &sinewave_bind_group_data.layout,
            ],
            ..Default::default()
        });

        let boundary_buffer_data = sine.boundary.create_buffer_data(device);

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Sine Pipeline"),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[boundary_buffer_data.vertex_buffer_layout.clone()],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    blend: Some(BlendState::REPLACE),
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
            boundary_buffer_data,
            global_bind_group_data,
            sinewave_bind_group_data,
            pipeline,
        }
    }

    pub(crate) fn set_render_pass(&self, render_pass: &mut RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.boundary_buffer_data.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            self.boundary_buffer_data.index_buffer.slice(..),
            IndexFormat::Uint16,
        );
        render_pass.set_bind_group(0, &self.global_bind_group_data.bind_group, &[]);
        render_pass.set_bind_group(1, &self.sinewave_bind_group_data.bind_group, &[]);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    pub(crate) fn update_global_frame(&mut self, queue: &Queue) {
        self.global.increment_frame();
        queue.write_buffer(
            &self.global_bind_group_data.buffer,
            0,
            bytemuck::bytes_of(&self.global),
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
