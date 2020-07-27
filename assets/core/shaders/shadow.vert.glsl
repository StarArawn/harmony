#version 450

layout(location = 0) in vec3 i_position;
layout(location = 0) out vec4 o_position;

layout(set = 0, binding = 0) uniform Locals {
    mat4 world;
};

layout(push_constant) uniform Uniforms {
    mat4 view_proj;
    vec4 light_pos;
};

void main() {
    o_position = world * vec4(i_position, 1.0);
    gl_Position = view_proj * world * vec4(i_position, 1.0);
}
