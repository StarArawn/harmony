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
};

layout (set = 1, binding = 1) uniform LightingData {
    uvec4 cluster_count;
    vec4 light_num; // Max depth stored in W.
    DirectionalLight directional_lights[4];
    PointLight point_lights[MAX_LIGHTS];
};

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