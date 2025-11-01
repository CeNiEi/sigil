use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Self { position: [x, y] }
    }
}
