use serde::Deserialize;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StateUniform {
    pub entities: [[f32; 4]; 64],
    pub count: u32,
    pub padding: [u32; 3],
}

impl Default for StateUniform {
    fn default() -> Self {
        Self {
            entities: [[0.0; 4]; 64],
            count: 0,
            padding: [0; 3],
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ServerMessage {
    PhysicsState { x: Vec<f32>, y: Vec<f32>, z: Vec<f32> },
    ShaderUpdate { wgsl: String },
    Error { detail: String },
}
