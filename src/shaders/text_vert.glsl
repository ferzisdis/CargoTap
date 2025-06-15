#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coords;

layout(location = 0) out vec2 frag_tex_coords;

layout(push_constant) uniform PushConstants {
    mat4 projection;
    vec2 screen_size;
} pc;

void main() {
    // Convert screen coordinates to normalized device coordinates
    vec2 normalized_pos = (position / pc.screen_size) * 2.0 - 1.0;
    normalized_pos.y = -normalized_pos.y; // Flip Y coordinate
    
    gl_Position = vec4(normalized_pos, 0.0, 1.0);
    frag_tex_coords = tex_coords;
}