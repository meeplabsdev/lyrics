struct VertexOut {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    @size(16) delta: f32, // pad to 16 bytes
    @size(16) time: f32, // pad to 16 bytes
    colours: array<vec4<f32>, 4>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0,  1.0), // top-left
    vec2<f32>(-1.0, -1.0), // bottom-left
    vec2<f32>( 1.0,  1.0), // top-right

    vec2<f32>( 1.0,  1.0), // top-right
    vec2<f32>(-1.0, -1.0), // bottom-left
    vec2<f32>( 1.0, -1.0)  // bottom-right
);

var<private> v_corner_indices: array<u32, 6> = array<u32, 6>(
    0u, // top-left
    1u, // bottom-left
    2u, // top-right

    2u, // top-right
    1u, // bottom-left
    3u  // bottom-right
);

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;

    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.color = uniforms.colours[v_corner_indices[v_idx]];

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}