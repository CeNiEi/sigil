use wgpu::{BindGroup, BindGroupLayout, Buffer, VertexBufferLayout};

pub(crate) struct BindGroupData {
    pub(crate) buffer: Buffer,
    pub(crate) layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
}

pub(crate) struct BufferData {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    pub(crate) vertex_buffer_layout: VertexBufferLayout<'static>,
}
