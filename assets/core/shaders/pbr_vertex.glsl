#version 450

layout(location = 0) in vec3 i_Pos;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec2 i_uv;
layout(location = 3) in vec4 i_tangent;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec3 o_normal;
layout(location = 2) out vec3 o_position;
layout(location = 3) out vec3 o_tangent;
layout(location = 4) out float o_tbn_handedness;
// layout(location = 5) out mat3 o_TBN;

layout(set = 1, binding = 0) uniform Globals {
    mat4 view_projection;
    mat4 camera_pos;
};

layout(set = 0, binding = 0) uniform Locals {
    mat4 world;
};

void main() {
    v_TexCoord = vec2(i_uv.x, 1.0-i_uv.y);
    mat3 normalMatrix = transpose(inverse(mat3(world)));
    o_normal = normalize(vec3(i_normal.x, i_normal.y, -i_normal.z)); //(world * vec4(i_normal, 0.0)).xyz;
    o_position = i_Pos;
    o_tangent = normalize(vec3(i_tangent.x, i_tangent.y, i_tangent.z)); //normalize(world * vec4(i_tangent.xyz, 0.0)).xyz;
    // vec3 bitangentW = cross(o_normal, o_tangent.xyz) * i_tangent.w;
    // o_TBN = mat3(o_tangent, bitangentW, o_normal);
    o_tbn_handedness = i_tangent.w;

    gl_Position = view_projection * vec4(o_position, 1.0);
}