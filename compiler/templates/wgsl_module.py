def build_wgsl_module(wgsl_expr: str, max_steps: int) -> str:
    return f"""
struct State {{
    entities: array<vec4<f32>, 64>,
    count: u32,
    padding1: u32,
    padding2: u32,
    padding3: u32,
}}

@group(0) @binding(0)
var<uniform> state: State;

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {{
    var out: VertexOutput;
    let x = f32((in_vertex_index << 1) & 2u);
    let y = f32(in_vertex_index & 2u);
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}}

fn opU(d1: vec2<f32>, d2: vec2<f32>) -> vec2<f32> {{
    if (d1.x < d2.x) {{ return d1; }}
    return d2;
}}

fn safe_pow(base: f32, exp: f32) -> f32 {{
    return pow(abs(base), exp);
}}

fn map(p: vec3<f32>) -> vec2<f32> {{
    let x = p.x;
    let y = p.y;
    let z = p.z;

    var final_dist = 1000000.0;
    let loop_count = max(1u, state.count);

    for (var i = 0u; i < loop_count; i = i + 1u) {{
        var state_x = 0.0;
        var state_y = 0.0;
        var state_z = 0.0;
        if (i < state.count) {{
            state_x = state.entities[i].x;
            state_y = state.entities[i].y;
            state_z = state.entities[i].z;
        }}

        let dist = {wgsl_expr};
        final_dist = min(final_dist, dist);
    }}

    return vec2<f32>(final_dist, 1.0);
}}

fn calcNormal(p: vec3<f32>) -> vec3<f32> {{
    let h = 0.001;
    let k = vec2<f32>(1.0, -1.0);
    return normalize(
        k.xyy * map(p + k.xyy * h).x +
        k.yyx * map(p + k.yyx * h).x +
        k.yxy * map(p + k.yxy * h).x +
        k.xxx * map(p + k.xxx * h).x
    );
}}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {{
    let uv = in.uv * 2.0 - 1.0;

    let ro = vec3<f32>(0.0, 0.0, 100.0);
    let rd = normalize(vec3<f32>(uv.x, uv.y, -1.5));
var t = 0.0;
for (var i = 0; i < {max_steps}; i = i + 1) {{
    let p = ro + rd * t;
    let res = map(p);
    let d = res.x;

    if (d < 0.001 * t) {{
        let n = calcNormal(p);
        let lightDir = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let diff = max(dot(n, lightDir), 0.1);

        var col = vec3<f32>(0.5, 0.7, 1.0) * diff;

        col = col * exp(-0.001 * t);

        return vec4<f32>(col, 1.0);
    }}
    t = t + d;
    if (t > 2000.0) {{
        break;
    }}
}}

    let sky = mix(vec3<f32>(0.02, 0.05, 0.1), vec3<f32>(0.1, 0.2, 0.3), uv.y * 0.5 + 0.5);
    return vec4<f32>(sky, 1.0);
}}
"""
