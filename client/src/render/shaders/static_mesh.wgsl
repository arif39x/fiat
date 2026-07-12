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

struct Material {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ambient_occlusion: f32,
};

struct InstanceData {
    model_matrix: mat4x4<f32>,
    material: Material,
};

@group(0) @binding(0) var<storage, read> instances: array<InstanceData>;

@group(1) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(1) var<uniform> light: Light;

struct VSInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) albedo: vec4<f32>,
    @location(4) metallic: f32,
    @location(5) roughness: f32,
    @location(6) ao: f32,
};

@vertex
fn vs_main(input: VSInput, @builtin(instance_index) instance_idx: u32) -> VSOutput {
    let inst = instances[instance_idx];
    let world_pos = inst.model_matrix * vec4<f32>(input.position, 1.0);
    let world_normal = (inst.model_matrix * vec4<f32>(input.normal, 0.0)).xyz;
    var output: VSOutput;
    output.position = camera.view_proj * world_pos;
    output.normal = normalize(world_normal);
    output.uv = input.uv;
    output.world_pos = world_pos.xyz;
    output.albedo = inst.material.albedo;
    output.metallic = inst.material.metallic;
    output.roughness = inst.material.roughness;
    output.ao = inst.material.ambient_occlusion;
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
    let n = normalize(input.normal);
    let v = normalize(camera.camera_pos.xyz - input.world_pos);
    let l = normalize(-light.direction);
    let h = normalize(v + l);

    let ndotl = max(dot(n, l), 0.0);
    let ndotv = max(dot(n, v), 0.0);
    let ndoth = max(dot(n, h), 0.0);
    let hdotv = max(dot(h, v), 0.0);

    let f0 = mix(vec3<f32>(0.04), input.albedo.rgb, input.metallic);
    let albedo = input.albedo.rgb * (1.0 - input.metallic);

    let d = distribution_ggx(ndoth, input.roughness);
    let g = geometry_smith(ndotv, ndotl, input.roughness);
    let f = fresnel_schlick(hdotv, f0);

    let specular = d * g * f / max(4.0 * ndotv * ndotl, 0.001);
    let diffuse = albedo / 3.14159;

    let ambient_term = light.ambient * albedo * input.ao;
    let diffuse_term = diffuse * light.color * ndotl;
    let specular_term = specular * light.color * ndotl;

    let final_color = ambient_term + (diffuse_term + specular_term) * (1.0 - input.ao * 0.5);
    return vec4<f32>(final_color, 1.0);
}
