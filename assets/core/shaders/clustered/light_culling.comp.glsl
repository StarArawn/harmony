#version 450

#include "frustum.glsl"
#include "../library/lighting.glsl"

// 1 light across 64 clusters.
// x is light
// y is cluster
layout (local_size_x = 1, local_size_y = 64, local_size_z = 1) in;

layout(set = 0, binding = 0) readonly buffer Frustums {
    Frustum frustums[];
};
layout(set = 0, binding = 1) buffer GlobalIndices {
    LightIndexSet light_index_list[];
};

void main() {
    uint cluster_index = gl_GlobalInvocationID.y;
    uvec3 cluster_coords = get_cluster_coords(cluster_index, cluster_count.xyz);
    uint frustum_index = get_frustum_list_index(cluster_coords.xy, cluster_count.xy);

    ZBounds z_bounds = get_zbounds(cluster_coords.z, cluster_count.z, light_num.w); // light_num.w is max depth.
    Frustum frustum = frustums[frustum_index];

    uint light_count = 0;
    // light_num.y is the point light count.
    for (uint i = 0; i < int(light_num.y); i++) {
        PointLight light = point_lights[i];
        Sphere sphere = Sphere(light.position.xyz, light.attenuation.x);

        if (contains_sphere(frustum, sphere) && contains_sphere(z_bounds, sphere)) {
            if (light_count < MAX_LIGHTS) {
                light_index_list[cluster_index].indices[light_count] = i;
                light_count += 1;
            } else {
                break;
            }
        }
    }

    light_index_list[cluster_index].count = light_count;
}