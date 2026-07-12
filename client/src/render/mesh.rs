use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bone_weights: [f32; 4],
    pub bone_indices: [u32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct StaticVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

pub fn create_cube() -> (Vec<StaticVertex>, Vec<u32>) {
    let vertices = vec![
        // Front face (z = 1)
        StaticVertex { position: [-0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] },
        StaticVertex { position: [0.5, -0.5, 0.5], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0] },
        StaticVertex { position: [0.5, 0.5, 0.5], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0] },
        StaticVertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0] },
        // Back face (z = -1)
        StaticVertex { position: [0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0] },
        StaticVertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0] },
        StaticVertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0] },
        StaticVertex { position: [0.5, 0.5, -0.5], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0] },
        // Right face (x = 1)
        StaticVertex { position: [0.5, -0.5, 0.5], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
        StaticVertex { position: [0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0] },
        StaticVertex { position: [0.5, 0.5, -0.5], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },
        StaticVertex { position: [0.5, 0.5, 0.5], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        // Left face (x = -1)
        StaticVertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0], uv: [0.0, 0.0] },
        StaticVertex { position: [-0.5, -0.5, 0.5], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0] },
        StaticVertex { position: [-0.5, 0.5, 0.5], normal: [-1.0, 0.0, 0.0], uv: [1.0, 1.0] },
        StaticVertex { position: [-0.5, 0.5, -0.5], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        // Top face (y = 1)
        StaticVertex { position: [-0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
        StaticVertex { position: [0.5, 0.5, 0.5], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0] },
        StaticVertex { position: [0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0] },
        StaticVertex { position: [-0.5, 0.5, -0.5], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0] },
        // Bottom face (y = -1)
        StaticVertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0] },
        StaticVertex { position: [0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0] },
        StaticVertex { position: [0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0] },
        StaticVertex { position: [-0.5, -0.5, 0.5], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0] },
    ];
    let indices = vec![
        0, 1, 2, 0, 2, 3,
        4, 5, 6, 4, 6, 7,
        8, 9, 10, 8, 10, 11,
        12, 13, 14, 12, 14, 15,
        16, 17, 18, 16, 18, 19,
        20, 21, 22, 20, 22, 23,
    ];
    (vertices, indices)
}

pub fn create_sphere(segments: u32) -> (Vec<StaticVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let radius = 0.5;

    for lat in 0..=segments {
        let theta = std::f32::consts::PI * lat as f32 / segments as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        for lon in 0..=segments {
            let phi = 2.0 * std::f32::consts::PI * lon as f32 / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;
            vertices.push(StaticVertex {
                position: [x * radius, y * radius, z * radius],
                normal: [x, y, z],
                uv: [lon as f32 / segments as f32, lat as f32 / segments as f32],
            });
        }
    }

    for lat in 0..segments {
        for lon in 0..segments {
            let first = (lat * (segments + 1) + lon);
            let second = first + segments + 1;
            indices.push(first);
            indices.push(second);
            indices.push(first + 1);
            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }

    (vertices, indices)
}

pub fn create_cylinder(segments: u32) -> (Vec<StaticVertex>, Vec<u32>) {
    let radius = 0.5;
    let height = 1.0;
    let half = height * 0.5;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=segments {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let ca = angle.cos();
        let sa = angle.sin();
        vertices.push(StaticVertex {
            position: [ca * radius, -half, sa * radius],
            normal: [ca, 0.0, sa],
            uv: [i as f32 / segments as f32, 0.0],
        });
        vertices.push(StaticVertex {
            position: [ca * radius, half, sa * radius],
            normal: [ca, 0.0, sa],
            uv: [i as f32 / segments as f32, 1.0],
        });
    }
    for i in 0..segments {
        let a = i * 2;
        let b = a + 1;
        let c = (i + 1) * 2;
        let d = c + 1;
        indices.push(a);
        indices.push(c);
        indices.push(b);
        indices.push(b);
        indices.push(c);
        indices.push(d);
    }
    (vertices, indices)
}

pub fn create_plane() -> (Vec<StaticVertex>, Vec<u32>) {
    let vertices = vec![
        StaticVertex { position: [-0.5, 0.0, -0.5], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
        StaticVertex { position: [0.5, 0.0, -0.5], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0] },
        StaticVertex { position: [0.5, 0.0, 0.5], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0] },
        StaticVertex { position: [-0.5, 0.0, 0.5], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0] },
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    (vertices, indices)
}


