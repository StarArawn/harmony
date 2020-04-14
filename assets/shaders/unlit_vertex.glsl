#version 450

layout(location = 0) in vec3 i_Pos;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec2 i_uv;
layout(location = 3) in vec4 i_tangent;

// layout(set = 0, binding = 0) uniform Locals {
//     mat4 u_Transform;
// };

void main() {
    // gl_Position = u_Transform * vec4(i_Pos, 1.0);
    gl_Position = vec4(i_Pos, 1.0);
}