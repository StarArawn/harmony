#version 450

#include "frustum.glsl"

layout (local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0) uniform Uniforms {
    Frustum frustum;
    mat4 inv_proj;
    uvec4 froxel_count;
};
layout(set = 0, binding = 1) buffer Frustums {
    Frustum result_frustums[];
};

void main() {
    vec2 lerp_start = vec2(gl_GlobalInvocationID) / vec2(froxel_count.xy);
    vec2 lerp_end = vec2(gl_GlobalInvocationID + 1) / vec2(froxel_count.xy);

    // clip space
    vec4 cs[4];
    // Top Left Plane
    cs[0] = vec4(mix(-1.0, 1.0, lerp_start.x), mix(-1.0, 1.0, lerp_start.y), 1.0, 1.0);
    // Top RightPlane
    cs[1] = vec4(mix(-1.0, 1.0, lerp_end.x), mix(-1.0, 1.0, lerp_start.y), 1.0, 1.0);
    // Bottom LeftPlane
    cs[2] = vec4(mix(-1.0, 1.0, lerp_start.x), mix(-1.0, 1.0, lerp_end.y), 1.0, 1.0);
    // Bottom RightPlane
    cs[3] = vec4(mix(-1.0, 1.0, lerp_end.x), mix(-1.0, 1.0, lerp_end.y), 1.0, 1.0);

    // view space
    vec3 vs[4];
    for (int i = 0; i < 4; ++i) {
        vec4 view = inv_proj * cs[i];
        view.xyz /= view.w;
        vs[i] = view.xyz;
    }

    vec3 eye = vec3(0.0);

    Frustum f;

    // Left
    f.planes[0] = compute_plane(eye, vs[0], vs[2]);
    // Right
    f.planes[1] = compute_plane(eye, vs[3], vs[1]);
    // Top
    f.planes[2] = compute_plane(eye, vs[1], vs[0]);
    // Bottom
    f.planes[3] = compute_plane(eye, vs[2], vs[3]);

    uint i = get_frustum_list_index(gl_GlobalInvocationID.xy, froxel_count.xy);

    result_frustums[i] = f;
}