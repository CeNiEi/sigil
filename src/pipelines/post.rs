use std::alloc::GlobalAlloc;

use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
    ColorTargetState, ColorWrites, Device, FilterMode, FragmentState, IndexFormat,
    MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, Queue,
    RenderPass, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor,
    ShaderStages, TextureFormat, TextureSampleType, TextureView, TextureViewDimension, VertexState,
    include_wgsl,
};

use crate::{
    boundary::Boundary,
    global::Global,
    utils::{BindGroupData, VertexBufferData},
};

pub(crate) struct PostPipeline {
    pipeline: RenderPipeline,
    off_screen_bind_group: BindGroup,
    global_bind_group_data: BindGroupData,
    global: Global,
}

impl PostPipeline {
    fn create_off_screen_bindgroup(
        texture_view: &TextureView,
        device: &Device,
    ) -> (BindGroupLayout, BindGroup) {
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Off Screen Texture View Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            label: Some("Off Screen Sampler"),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Off Screen Bind Group"),
            layout: &layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        (layout, bind_group)
    }

    pub(crate) fn new(
        texture_view: &TextureView,
        texture_format: TextureFormat,
        global: Global,
        device: &Device,
    ) -> Self {
        let (off_screen_bind_group_layout, off_screen_bind_group) =
            Self::create_off_screen_bindgroup(texture_view, device);

        let global_bind_group_data = global.create_bind_group_data(device);

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Post Pipeline Layout"),
            bind_group_layouts: &[
                &off_screen_bind_group_layout,
                &global_bind_group_data.layout,
            ],
            ..Default::default()
        });

        let shader_module = device.create_shader_module(include_wgsl!("post.wgsl"));

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Post Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multiview: None,
            cache: None,
            multisample: MultisampleState::default(),
        });

        Self {
            pipeline,
            off_screen_bind_group,
            global_bind_group_data,
            global,
        }
    }

    pub(crate) fn set_render_pass(&self, render_pass: &mut RenderPass<'_>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.off_screen_bind_group, &[]);
        render_pass.set_bind_group(1, &self.global_bind_group_data.bind_group, &[]);

        render_pass.draw(0..6, 0..1);
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

    pub(crate) fn update_off_screen_bindgroup(
        &mut self,
        texture_view: &TextureView,
        device: &Device,
    ) {
        let (_, bing_group) = Self::create_off_screen_bindgroup(texture_view, device);

        self.off_screen_bind_group = bing_group;
    }
}
