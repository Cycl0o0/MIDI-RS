// Note rendering shader for MIDI-RS Black MIDI Visualizer
// Uses instanced rendering for efficient note display

// Uniforms passed from CPU
struct Uniforms {
    screen_size: vec2<f32>,
    playhead_position: f32,
    time: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Vertex input (quad vertices)
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

// Instance input (per-note data)
struct InstanceInput {
    @location(2) instance_position: vec2<f32>,
    @location(3) instance_size: vec2<f32>,
    @location(4) instance_color: vec4<f32>,
}

// Vertex output to fragment shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) world_position: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Calculate the world position of this vertex
    // vertex.position is 0-1 for the quad corners
    // instance_position is the note's position (0-1 normalized screen space)
    // instance_size is the note's size in normalized coordinates
    let world_pos = instance.instance_position + vertex.position * instance.instance_size;
    
    // Convert to clip space (-1 to 1)
    // Y is inverted so that higher pitch = higher on screen
    let clip_x = world_pos.x * 2.0 - 1.0;
    let clip_y = world_pos.y * 2.0 - 1.0;
    
    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.color = instance.instance_color;
    out.tex_coords = vertex.tex_coords;
    out.world_position = world_pos;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple flat color rendering for performance
    var color = in.color;
    
    // Optional: Add slight glow effect based on distance from playhead
    let playhead_distance = abs(in.world_position.x - uniforms.playhead_position);
    let glow_factor = 1.0 - smoothstep(0.0, 0.05, playhead_distance);
    
    // Brighten notes near the playhead
    color = vec4<f32>(
        min(color.r + glow_factor * 0.3, 1.0),
        min(color.g + glow_factor * 0.3, 1.0),
        min(color.b + glow_factor * 0.3, 1.0),
        color.a
    );
    
    // Optional: Fade notes that are far from the playhead
    let fade_start = 0.7;
    let fade_end = 0.95;
    if in.world_position.x > fade_start {
        let fade_factor = 1.0 - smoothstep(fade_start, fade_end, in.world_position.x);
        color.a *= fade_factor;
    }
    
    return color;
}
