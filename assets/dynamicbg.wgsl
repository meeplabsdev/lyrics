// Kawarp shaders ported from GLSL (WebGL1) to WGSL.
// Source: better-lyrics/kawarp packages/core/src/index.ts

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    var out: VertexOut;
    let p = positions[v_idx];
    out.position = vec4<f32>(p, 0.0, 1.0);
    out.tex_coord = p * 0.5 + vec2<f32>(0.5);

    return out;
}

@vertex
fn vs_offscreen(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );

    var out: VertexOut;
    let p = positions[v_idx];
    out.position = vec4<f32>(p, 0.0, 1.0);
    out.tex_coord = vec2<f32>(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5);

    return out;
}

struct BlurUniforms {
    resolution: vec2<f32>,
    offset: f32,
};

@group(0) @binding(0) var<uniform> blur_u: BlurUniforms;
@group(0) @binding(1) var blur_samp: sampler;
@group(0) @binding(2) var blur_tex: texture_2d<f32>;

@fragment
fn fs_blur(in: VertexOut) -> @location(0) vec4<f32> {
    let texel_size = 1.0 / blur_u.resolution;
    let o = blur_u.offset;
    var color = vec4<f32>(0.0);
    color += textureSampleLevel(blur_tex, blur_samp, in.tex_coord + vec2<f32>(-o, -o) * texel_size, 0.0);
    color += textureSampleLevel(blur_tex, blur_samp, in.tex_coord + vec2<f32>( o, -o) * texel_size, 0.0);
    color += textureSampleLevel(blur_tex, blur_samp, in.tex_coord + vec2<f32>(-o,  o) * texel_size, 0.0);
    color += textureSampleLevel(blur_tex, blur_samp, in.tex_coord + vec2<f32>( o,  o) * texel_size, 0.0);

    return color * 0.25;
}

struct BlendUniforms {
    blend: f32,
};

@group(0) @binding(0) var<uniform> blend_u: BlendUniforms;
@group(0) @binding(1) var blend_samp: sampler;
@group(0) @binding(2) var blend_tex1: texture_2d<f32>;
@group(0) @binding(3) var blend_tex2: texture_2d<f32>;

@fragment
fn fs_blend(in: VertexOut) -> @location(0) vec4<f32> {
    let color1 = textureSampleLevel(blend_tex1, blend_samp, in.tex_coord, 0.0);
    let color2 = textureSampleLevel(blend_tex2, blend_samp, in.tex_coord, 0.0);

    return mix(color1, color2, blend_u.blend);
}

struct TintUniforms {
    tint_color: vec3<f32>,
    tint_intensity: f32,
};

@group(0) @binding(0) var<uniform> tint_u: TintUniforms;
@group(0) @binding(1) var tint_samp: sampler;
@group(0) @binding(2) var tint_tex: texture_2d<f32>;

@fragment
fn fs_tint(in: VertexOut) -> @location(0) vec4<f32> {
    var color = textureSampleLevel(tint_tex, tint_samp, in.tex_coord, 0.0);
    let luma = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let dark_mask = 1.0 - smoothstep(0.0, 0.5, luma);
    let rgb = mix(color.rgb, tint_u.tint_color, dark_mask * tint_u.tint_intensity);

    return vec4<f32>(rgb, color.a);
}

struct WarpUniforms {
    time: f32,
    intensity: f32,
};

@group(0) @binding(0) var<uniform> warp_u: WarpUniforms;
@group(0) @binding(1) var warp_samp: sampler;
@group(0) @binding(2) var warp_tex: texture_2d<f32>;

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_2(x: vec2<f32>) -> vec2<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec3<f32>) -> vec3<f32> {
    return mod289_3(((x * 34.0) + 1.0) * x);
}

fn snoise(v: vec2<f32>) -> f32 {
    let C = vec4<f32>(
        0.211324865405187,
        0.366025403784439,
        -0.577350269189626,
        0.024390243902439
    );

    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    let i1 = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), x0.x > x0.y);

    var x12 = x0.xyxy + C.xxzz;
    x12 = vec4<f32>(x12.xy - i1, x12.zw);

    i = mod289_2(i);
    let p = permute(
        permute(vec3<f32>(i.y) + vec3<f32>(0.0, i1.y, 1.0))
        + vec3<f32>(i.x) + vec3<f32>(0.0, i1.x, 1.0)
    );

    var m = max(
        vec3<f32>(0.5) - vec3<f32>(
            dot(x0, x0),
            dot(x12.xy, x12.xy),
            dot(x12.zw, x12.zw)
        ),
        vec3<f32>(0.0)
    );
    m = m * m;
    m = m * m;

    let x = 2.0 * fract(p * C.www) - 1.0;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    let g = vec3<f32>(
        a0.x * x0.x + h.x * x0.y,
        a0.y * x12.x + h.y * x12.y,
        a0.z * x12.z + h.z * x12.w
    );

    return 130.0 * dot(m, g);
}

@fragment
fn fs_warp(in: VertexOut) -> @location(0) vec4<f32> {
    let uv = in.tex_coord;
    let t = warp_u.time * 0.05;

    let center = uv - 0.5;
    let center_weight = 1.0 - smoothstep(0.0, 0.7, length(center));

    let n1 = snoise(uv * 0.35 + vec2<f32>(t, t * 0.7));
    let n2 = snoise(uv * 0.35 + vec2<f32>(-t * 0.8, t * 0.5) + vec2<f32>(50.0, 50.0));
    let n3 = snoise(uv * 0.9 + vec2<f32>(t * 1.2, -t) + vec2<f32>(100.0, 0.0));
    let n4 = snoise(uv * 0.9 + vec2<f32>(-t, t * 1.1) + vec2<f32>(0.0, 100.0));

    let warp = vec2<f32>(
        n1 * 0.65 + n3 * 0.35,
        n2 * 0.65 + n4 * 0.35
    ) * center_weight;

    var warped_uv = uv + warp * warp_u.intensity;
    warped_uv = clamp(warped_uv, vec2<f32>(0.0), vec2<f32>(1.0));

    return textureSampleLevel(warp_tex, warp_samp, warped_uv, 0.0);
}

struct OutputUniforms {
    saturation: f32,
    dithering: f32,
    time: f32,
    scale: f32,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> out_u: OutputUniforms;
@group(0) @binding(1) var out_samp: sampler;
@group(0) @binding(2) var out_tex: texture_2d<f32>;

fn hash(p_in: vec3<f32>) -> f32 {
    var p = fract(p_in * 0.1031);
    p = p + vec3<f32>(dot(p, p.zyx + vec3<f32>(31.32)));
    
    return fract((p.x + p.y) * p.z);
}

@fragment
fn fs_output(in: VertexOut) -> @location(0) vec4<f32> {
    var uv = (in.tex_coord - 0.5) / out_u.scale + 0.5;
    uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));

    var color = textureSampleLevel(out_tex, out_samp, uv, 0.0);

    let center = in.tex_coord - 0.5;
    let vignette = 1.0 - dot(center, center) * 0.3;
    color = vec4<f32>(color.rgb * vignette, color.a);

    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    color = vec4<f32>(mix(vec3<f32>(gray), color.rgb, out_u.saturation), color.a);

    let pixel_pos = floor(in.tex_coord * out_u.resolution);
    let noise = hash(vec3<f32>(pixel_pos, floor(out_u.time * 60.0)));
    color = vec4<f32>(color.rgb + (noise - 0.5) * out_u.dithering, color.a);

    return color;
}

struct DownsampleUniforms {
    texel_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> down_u: DownsampleUniforms;
@group(0) @binding(1) var down_samp: sampler;
@group(0) @binding(2) var down_tex: texture_2d<f32>;

@fragment
fn fs_downsample(in: VertexOut) -> @location(0) vec4<f32> {
    let offset = down_u.texel_size * 0.25;
    return 0.25 * (
        textureSampleLevel(down_tex, down_samp, in.tex_coord + vec2<f32>(-offset.x, -offset.y), 0.0) +
        textureSampleLevel(down_tex, down_samp, in.tex_coord + vec2<f32>( offset.x, -offset.y), 0.0) +
        textureSampleLevel(down_tex, down_samp, in.tex_coord + vec2<f32>(-offset.x,  offset.y), 0.0) +
        textureSampleLevel(down_tex, down_samp, in.tex_coord + vec2<f32>( offset.x,  offset.y), 0.0)
    );
}

struct SmoothingUniforms {
    response: f32,
    min_step: f32,
};

@group(0) @binding(0) var<uniform> smooth_u: SmoothingUniforms;
@group(0) @binding(1) var smooth_samp: sampler;
@group(0) @binding(2) var smooth_tex: texture_2d<f32>;
@group(0) @binding(3) var smooth_history: texture_2d<f32>;

@fragment
fn fs_smoothing(in: VertexOut) -> @location(0) vec4<f32> {
    let history = textureSampleLevel(smooth_history, smooth_samp, in.tex_coord, 0.0);
    let current = textureSampleLevel(smooth_tex, smooth_samp, in.tex_coord, 0.0);

    let blended = mix(history, current, smooth_u.response);

    let remaining = current - history;
    let minimal = history + sign(remaining) * min(abs(remaining), vec4<f32>(smooth_u.min_step));

    return mix(
        minimal,
        blended,
        step(abs(minimal - history), abs(blended - history))
    );
}
