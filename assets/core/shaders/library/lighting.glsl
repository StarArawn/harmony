#ifndef LIGHTING_INCLUDES
#define LIGHTING_INCLUDES

const int MAX_LIGHTS = 10;

struct DirectionalLight {
    vec3 direction;
    vec3 color;
};

struct PointLight {
    vec3 position;
    vec3 color;
    vec3 attenuation;
};

layout (set = 1, binding = 1) uniform LightingData {
    vec4 light_num;
    DirectionalLight directional_lights[MAX_LIGHTS / 2];
    PointLight point_lights[MAX_LIGHTS / 2];
};

DirectionalLight get_directional_light(int index) {
    return directional_lights[index];
}

#endif