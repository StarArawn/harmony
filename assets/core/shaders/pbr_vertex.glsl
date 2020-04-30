#version 450

#include "library/common.glsl"

layout(location = 0) in vec3 i_Pos;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec2 i_uv;
layout(location = 3) in vec4 i_tangent;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec3 o_normal;
layout(location = 2) out vec3 o_position;
layout(location = 3) out vec3 o_tangent;
layout(location = 4) out float o_tbn_handedness;

layout(set = 0, binding = 0) uniform Locals {
    mat4 world;
};

void main() {
    v_TexCoord = vec2(i_uv.x, i_uv.y);
    mat3 normalMatrix = mat3(transpose(inverse(world)));
    o_position = (world * vec4(i_Pos, 1.0)).xyz;
    o_normal = normalize(world * vec4(i_normal.xyz, 0.0)).xyz;
    o_tangent = normalize(world * vec4(i_tangent.xyz, 0.0)).xyz;
    o_tbn_handedness = i_tangent.w;
    gl_Position = view_projection * world * vec4(i_Pos, 1.0);
}