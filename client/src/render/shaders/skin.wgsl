struct SkinUniform {
    joint_count: u32,
    padding: u32,
};

struct JointMatrix {
    data: array<mat4x4<f32>, 128>,
};

struct Camera {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
};

struct Light {
    direction: vec3<f32>,
    padding: f32,
    color: vec3<f32>,
    ambient: vec3<f32>,
};

@group(0) @binding(0) var<uniform> skin: SkinUniform;
@group(0) @binding(1) var<uniform> joint_matrices: JointMatrix;

@group(1) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(1) var texture_sampler: sampler;
@group(1) @binding(2) var texture: texture_2d<f32>;
@group(1) @binding(3) var<uniform> light: Light;

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
    output.position = camera.view_proj * world_pos;
    output.normal = normalize(world_normal);
    output.uv = input.uv;
    output.world_pos = world_pos.xyz;
    return output;
}

fn distribution_ggx(ndoth: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = ndoth * ndoth * (a2 - 1.0) + 1.0;
    return a2 / (3.14159 * denom * denom);
}

fn geometry_smith(ndotv: f32, ndotl: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let k = (a + 1.0) * (a + 1.0) / 8.0;
    let g1 = ndotl / (ndotl * (1.0 - k) + k);
    let g2 = ndotv / (ndotv * (1.0 - k) + k);
    return g1 * g2;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

@fragment
fn fs_main(input: VSOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, input.uv).rgb;
    let n = normalize(input.normal);
    let v = normalize(camera.camera_pos.xyz - input.world_pos);
    let l = normalize(-light.direction);
    let h = normalize(v + l);

    let ndotl = max(dot(n, l), 0.0);
    let ndotv = max(dot(n, v), 0.0);
    let ndoth = max(dot(n, h), 0.0);
    let hdotv = max(dot(h, v), 0.0);

    let roughness = 0.5;
    let metallic = 0.0;
    let f0 = mix(vec3<f32>(0.04), tex_color, metallic);
    let albedo = tex_color * (1.0 - metallic);

    let d = distribution_ggx(ndoth, roughness);
    let g = geometry_smith(ndotv, ndotl, roughness);
    let f = fresnel_schlick(hdotv, f0);

    let specular = d * g * f / max(4.0 * ndotv * ndotl, 0.001);
    let diffuse = albedo / 3.14159;

    let ambient_term = light.ambient * albedo;
    let diffuse_term = diffuse * light.color * ndotl;
    let specular_term = specular * light.color * ndotl;

    let final_color = ambient_term + diffuse_term + specular_term;
    return vec4<f32>(final_color, 1.0);
}
