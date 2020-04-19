#ifndef LIGHTING_INCLUDES
#define LIGHTING_INCLUDES

const int MAX_LIGHTS = 10;

layout(std140) struct DirectionalLight {
    vec3 direction;
    vec3 color;
};

layout(std140) struct PointLight {
    vec3 position;
    float attenuation;
    vec3 color;
};

layout (std140, set = 1, binding = 1) uniform LightingData {
    int TOTAL_DIRECTIONAL_LIGHTS;
    int TOTAL_POINT_LIGHTS;
    DirectionalLight directional_lights[MAX_LIGHTS / 2];
    PointLight point_lights[MAX_LIGHTS / 2];
};

DirectionalLight get_directional_light(int index) {
    return directional_lights[index];
}

#endif