struct PBRUniforms {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ambient_occlusion: f32,
    padding: f32,
};

struct Light {
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    padding: f32,
};

@group(0) @binding(0) var<uniform> material: PBRUniforms;
@group(0) @binding(1) var<uniform> light: Light;
@group(0) @binding(2) var texture_sampler: sampler;
@group(0) @binding(3) var albedo_texture: texture_2d<f32>;

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
};

@fragment
fn fs_main(input: VSOutput) -> @location(0) vec4<f32> {
    let albedo = material.albedo.rgb * textureSample(albedo_texture, texture_sampler, input.uv).rgb;

    let N = normalize(input.normal);
    let L = normalize(-light.direction);
    let V = normalize(camera_position - input.world_pos);
    let H = normalize(L + V);

    let ndotl = max(dot(N, L), 0.0);
    let ndoth = max(dot(N, H), 0.0);
    let ndotv = max(dot(N, V), 0.0);

    let roughness = max(material.roughness, 0.01);
    let alpha = roughness * roughness;
    let d = ndoth * ndoth * (alpha * alpha - 1.0) + 1.0;
    let ndf = alpha * alpha / (3.14159 * d * d);

    let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let visibility = ndotl * (1.0 - k) + k;
    let gsf = ndotv * (1.0 - k) + k;
    let g = 0.25 / (visibility * gsf);

    let f0 = mix(vec3<f32>(0.04), albedo, material.metallic);
    let fresnel = f0 + (1.0 - f0) * pow(1.0 - ndoth, 5.0);

    let specular = ndf * g * fresnel;

    let kd = (1.0 - fresnel) * (1.0 - material.metallic);
    let diffuse = albedo / 3.14159 * kd;

    let ambient = material.ambient_occlusion * 0.03 * albedo;
    let lo = (diffuse + specular) * light.color * light.intensity * ndotl;

    return vec4<f32>(ambient + lo, 1.0);
}
