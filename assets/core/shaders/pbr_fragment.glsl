#version 450
#extension GL_GOOGLE_include_directive : enable

layout(location = 0) in vec2 v_TexCoord;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec4 i_position;
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
    for (int i=0; i < int(light_num.x) && i < MAX_LIGHTS; ++i) {
        DirectionalLight light = get_directional_light(i);
        float dot_product =  dot(normalize(i_normal), light.direction.xyz);
        color += dot_product * light.color.xyz;
    }

    // TODO: calculate attenuation.
    for (int i=0; i < int(light_num.y) && i < MAX_LIGHTS; ++i) {
        PointLight light = get_point_light(i);
        vec3 light_dir = normalize(light.position.xyz - i_position.xyz);
        float dot_product =  dot(normalize(i_normal), light_dir);
        color += dot_product * light.color.xyz;
    }

    outColor = vec4(color * tex.xyz, tex.w);
}