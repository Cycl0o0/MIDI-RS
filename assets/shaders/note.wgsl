// Note rendering shader for MIDI-RS Black MIDI Visualizer
// Uses instanced rendering for efficient note display
// Notes fall from top to bottom (like PFA - Piano From Above)

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
    // X maps pitch to horizontal position (0=left, 1=right)
    // Y maps time to vertical position (notes fall from top to bottom)
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
    
    // Add slight glow effect based on distance from playhead (near bottom)
    let playhead_y = 0.15; // Playhead is at 15% from bottom
    let playhead_distance = abs(in.world_position.y - playhead_y);
    let glow_factor = 1.0 - smoothstep(0.0, 0.05, playhead_distance);
    
    // Brighten notes near the playhead
    color = vec4<f32>(
        min(color.r + glow_factor * 0.3, 1.0),
        min(color.g + glow_factor * 0.3, 1.0),
        min(color.b + glow_factor * 0.3, 1.0),
        color.a
    );
    
    // Fade notes at the top of the screen (far future)
    let fade_start = 0.85;
    let fade_end = 0.98;
    if in.world_position.y > fade_start {
        let fade_factor = 1.0 - smoothstep(fade_start, fade_end, in.world_position.y);
        color.a *= fade_factor;
    }
    
    // Slight fade for notes below the playhead (past notes)
    if in.world_position.y < playhead_y {
        let past_fade = smoothstep(0.0, playhead_y, in.world_position.y);
        color.a *= past_fade * 0.5 + 0.5;
    }
    
    return color;
}
