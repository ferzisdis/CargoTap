#version 450

layout(location = 0) in vec2 frag_tex_coords;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform sampler2D glyph_texture;

layout(push_constant) uniform PushConstants {
    mat4 projection;
    vec2 screen_size;
    vec4 text_color;
} pc;

void main() {
    float alpha = texture(glyph_texture, frag_tex_coords).r;
    f_color = vec4(pc.text_color.rgb, pc.text_color.a * alpha);
    
    // Discard fully transparent pixels
    if (f_color.a < 0.01) {
        discard;
    }
}