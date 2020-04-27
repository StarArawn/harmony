#version 450

layout(location = 0) in vec3 i_Pos;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec2 i_uv;
layout(location = 3) in vec4 i_tangent;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec3 o_normal;
layout(location = 2) out vec4 o_position;
layout(location = 3) out mat3 o_TBN;
// layout(location = 3) out vec3 o_tangent;
// layout(location = 4) out float o_tbn_handedness;

layout(set = 1, binding = 0) uniform Globals {
    mat4 view_projection;
    mat4 camera_pos;
};

layout(set = 0, binding = 0) uniform Locals {
    mat4 world;
};

void main() {
    v_TexCoord = i_uv;
    mat3 normalMatrix = transpose(inverse(mat3(world)));
    o_normal = normalize(normalMatrix * i_normal.xyz);
    o_position = world * vec4(i_Pos, 1.0);
    // o_tangent = normalize(transpose(inverse(mat3(world))) * i_tangent.xyz); //normalize((world * vec4(i_tangent.xyz, 0.0)).xyz);
    // o_tbn_handedness = i_tangent.w;

    // vec3 N = normalize(o_normal);
    // vec3 T = normalize(i_tangent.xyz - N * dot(N, i_tangent.xyz));
    // vec3 B = normalize(cross(N, T)) * i_tangent.w;
    // o_TBN = normalMatrix * mat3(T, B, N);
    vec3 tbnNormal = normalize(o_normal);
    vec3 tbnTangent = normalize(i_tangent.xyz);
    vec3 tbnBitangent = cross(tbnNormal, tbnTangent) * i_tangent.w;
    o_TBN = normalMatrix * mat3(tbnTangent, tbnBitangent, tbnNormal);

    gl_Position = view_projection * o_position;
}