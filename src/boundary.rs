use wgpu::{
    Buffer, BufferUsages, Device, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexStepMode,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{utils::VertexBufferData, vertex::Vertex};

pub(crate) struct Boundary {
    inner: [Vertex; 4],
}

impl Boundary {
    pub(crate) fn new(tl: Vertex, bl: Vertex, br: Vertex, tr: Vertex) -> Self {
        Self {
            inner: [tl, bl, br, tr],
        }
    }

    fn vertices(&self) -> [Vertex; 4] {
        self.inner
    }

    fn indices() -> [u16; 6] {
        [0, 1, 3, 1, 2, 3]
    }

    pub(crate) fn create_vertex_buffer_data(&self, device: &Device) -> VertexBufferData {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Boundary Vertex Buffer"),
            contents: bytemuck::bytes_of(&self.vertices()),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Boundary Index Buffer"),
            contents: bytemuck::cast_slice(&Self::indices()),
            usage: BufferUsages::INDEX,
        });

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x2,
                shader_location: 0,
                offset: 0,
            }],
        };

        VertexBufferData {
            vertex_buffer,
            vertex_buffer_layout,
            index_buffer,
        }
    }
}
