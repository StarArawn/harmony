#version 450

layout(location = 0) in vec3 i_Pos;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec2 i_uv;
layout(location = 3) in vec4 i_tangent;

layout(set = 0, binding = 0) uniform Locals {
    mat4 view_projection;
};

void main() {
    gl_Position = view_projection * vec4(i_Pos, 1.0);
}