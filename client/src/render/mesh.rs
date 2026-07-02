pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bone_weights: [f32; 4],
    pub bone_indices: [u32; 4],
}

pub struct SkinnedMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct SkinBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}
