#version 450

layout(location = 0) in vec2 v_TexCoord;
layout(location = 0) out vec4 outColor;

layout(set = 2, binding = 0) uniform Locals {
    vec3 color;
};
// layout(set = 2, binding = 1) uniform texture2D t_Color;
// layout(set = 2, binding = 2) uniform sampler s_Color;

void main() {
    // vec4 tex = texture(sampler2D(t_Color, s_Color), v_TexCoord);
    outColor = vec4(1, 0, 0, 1);
}