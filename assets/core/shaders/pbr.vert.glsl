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
layout(location = 5) out vec4 o_clip_position;
layout(location = 6) out vec4 o_view_position;
layout(location = 7) out vec3 o_vertex;

layout(set = 0, binding = 0) uniform Locals {
    mat4 world;
};

void main() {
    o_vertex = i_Pos;
    v_TexCoord = vec2(i_uv.x, i_uv.y);
    mat3 normalMatrix = mat3(transpose(inverse(world)));
    o_position = (world * vec4(i_Pos, 1.0)).xyz;
    o_normal = normalMatrix * i_normal.xyz;
    o_tangent = normalMatrix * i_tangent.xyz;
    o_tbn_handedness = i_tangent.w;
    o_view_position = view * world * vec4(i_Pos, 1.0);
    vec4 clip_space = projection * view * world * vec4(i_Pos, 1.0);
    gl_Position = clip_space;
    o_clip_position = clip_space;
}