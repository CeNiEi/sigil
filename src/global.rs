use std::num::NonZero;

use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, BufferUsages, Device, ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::utils::BindGroupData;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct Global {
    resolution: [f32; 2],
    phase: f32,
    _padding: f32,
}

impl Global {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self {
            resolution: [width as f32, height as f32],
            phase: 0.,
            _padding: 0.,
        }
    }

    pub(crate) fn increment_frame(&mut self) {
        self.phase = self.phase + 1.;
    }

    pub(crate) fn set_resolution(&mut self, width: u32, height: u32) {
        self.resolution = [width as f32, height as f32];
    }

    pub(crate) fn create_bind_group_data(&self, device: &Device) -> BindGroupData {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: bytemuck::bytes_of(self),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Global Bind Group Layout"),
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
            label: Some("Global Bind Group"),
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
