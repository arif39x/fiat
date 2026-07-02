struct SkinUniform {
    joint_count: u32,
    padding: u32,
};

struct JointMatrix {
    data: array<mat4x4<f32>, 128>,
};

@group(0) @binding(0) var<uniform> skin: SkinUniform;
@group(0) @binding(1) var<uniform> joint_matrices: JointMatrix;

struct VSInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) bone_weights: vec4<f32>,
    @location(4) bone_indices: vec4<u32>,
};

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
};

@vertex
fn vs_main(input: VSInput) -> VSOutput {
    var skin_matrix = mat4x4<f32>(
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 0.0),
    );

    for (var i = 0u; i < 4u; i++) {
        let joint_idx = input.bone_indices[i];
        let weight = input.bone_weights[i];
        if joint_idx < 128u && weight > 0.0 {
            skin_matrix = skin_matrix + joint_matrices.data[joint_idx] * weight;
        }
    }

    let world_pos = skin_matrix * vec4<f32>(input.position, 1.0);
    let world_normal = (skin_matrix * vec4<f32>(input.normal, 0.0)).xyz;

    var output: VSOutput;
    output.position = world_pos;
    output.normal = normalize(world_normal);
    output.uv = input.uv;
    output.world_pos = world_pos.xyz;
    return output;
}

struct Light {
    direction: vec3<f32>,
    color: vec3<f32>,
    ambient: vec3<f32>,
};

@group(1) @binding(0) var<uniform> light: Light;
@group(1) @binding(1) var texture_sampler: sampler;
@group(1) @binding(2) var texture: texture_2d<f32>;

@fragment
fn fs_main(input: VSOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, input.uv).rgb;
    let ndotl = max(dot(input.normal, normalize(-light.direction)), 0.0);
    let diffuse = light.color * ndotl;
    let final_color = tex_color * (light.ambient + diffuse);
    return vec4<f32>(final_color, 1.0);
}
