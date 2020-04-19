#version 450
#extension GL_GOOGLE_include_directive : enable

layout(location = 0) in vec2 v_TexCoord;
layout(location = 1) in vec3 i_normal;
layout(location = 0) out vec4 outColor;

layout(set = 2, binding = 0) uniform Locals {
    vec3 color;
};
layout(set = 2, binding = 1) uniform texture2D t_Color;
layout(set = 2, binding = 2) uniform sampler s_Color;

#include "library/lighting.glsl"

void main() {
    vec4 tex = texture(sampler2D(t_Color, s_Color), v_TexCoord);
    vec3 ambient = vec3(0.05, 0.05, 0.05);
    // accumulate color
    vec3 color = ambient;
    for (int i=0; i < TOTAL_DIRECTIONAL_LIGHTS && i < MAX_LIGHTS; ++i) {
        DirectionalLight light = get_directional_light(i);
        float dot_product =  max(0.0, dot(normalize(light.direction), normalize(i_normal)));
        color += dot_product * light.color;
    }
    DirectionalLight light = get_directional_light(0);
    outColor = vec4(light.color, 1.0); //vec4(color * tex.xyz, tex.w);
}