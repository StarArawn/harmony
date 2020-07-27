#ifndef LIGHTING_INCLUDES
#define LIGHTING_INCLUDES

const int MAX_LIGHTS = 16;
const int MAX_LIGHTS_PER_CLUSTER = 128;

struct DirectionalLight {
    vec4 direction;
    vec4 color;
};

struct PointLight {
    vec4 position;
    vec4 view_position; // Used to cull lights in view space.
    vec4 color;
    vec4 attenuation; 
    mat4 shadow_matrix;
};

layout (set = 1, binding = 1) uniform LightingData {
    uvec4 cluster_count;
    vec4 light_num; // Max depth stored in W.
    DirectionalLight directional_lights[4];
    PointLight point_lights[MAX_LIGHTS];
};

layout(set = 1, binding = 4) uniform samplerShadow shadow_sampler;
layout(set = 1, binding = 5) uniform textureCubeArray omni_shadow_quad_1;
layout(set = 1, binding = 6) uniform textureCubeArray omni_shadow_quad_2;
layout(set = 1, binding = 7) uniform textureCubeArray omni_shadow_quad_3;
layout(set = 1, binding = 8) uniform textureCubeArray omni_shadow_quad_4;

DirectionalLight get_directional_light(int index) {
    return directional_lights[index];
}

PointLight get_point_light(int index) {
    return point_lights[index];
}

struct LightIndexSet {
    uint count;
    uint indices[MAX_LIGHTS_PER_CLUSTER - 1];
};

#endif