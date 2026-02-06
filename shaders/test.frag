#version 450 core

const vec2 RESOLUTION = vec2(1280, 720);

layout(binding = 0) uniform sampler2D tex;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = texture(tex, gl_FragCoord.xy / RESOLUTION);
}