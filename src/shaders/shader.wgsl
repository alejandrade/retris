// WGSL uniform buffers need std140 layout alignment
// Each vec4 takes 16 bytes, so we'll use vec4s
struct Uniforms {
    cube_pos: vec2<f32>,      // cube_x, cube_y (8 bytes, but aligned to 16)
    window_size: vec2<f32>,   // window_width, window_height
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    // Convert from pixel coordinates to normalized device coordinates (NDC)
    // wgpu uses NDC where (-1, -1) is bottom-left and (1, 1) is top-right
    // model.position is in local space (centered at origin), uniforms.cube_pos is the center position
    let world_x = model.position.x + uniforms.cube_pos.x;
    let world_y = model.position.y + uniforms.cube_pos.y;
    
    // Convert to NDC: map [0, window_width] to [-1, 1] for x
    // and [0, window_height] to [1, -1] for y (flip y-axis to handle Y-down screen coordinates)
    let x_ndc = (world_x / uniforms.window_size.x) * 2.0 - 1.0;
    let y_ndc = 1.0 - (world_y / uniforms.window_size.y) * 2.0;
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(x_ndc, y_ndc, 0.0, 1.0);
    out.color = model.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
