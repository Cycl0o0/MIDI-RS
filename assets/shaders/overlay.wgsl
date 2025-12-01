// Overlay shader for MIDI-RS Black MIDI Visualizer
// Simple shader for UI elements and text rendering

// Uniforms
struct Uniforms {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// Instance input for overlay elements
struct InstanceInput {
    @location(2) instance_position: vec2<f32>,
    @location(3) instance_size: vec2<f32>,
    @location(4) instance_color: vec4<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Calculate world position
    let world_pos = instance.instance_position + vertex.position * instance.instance_size;
    
    // Convert to clip space
    let clip_x = world_pos.x * 2.0 - 1.0;
    let clip_y = 1.0 - world_pos.y * 2.0; // Flip Y for screen coordinates
    
    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.color = instance.instance_color;
    out.tex_coords = vertex.tex_coords;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple solid color for overlay elements
    return in.color;
}
