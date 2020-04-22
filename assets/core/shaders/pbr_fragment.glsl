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

layout(set = 3, binding = 0) uniform textureCube spec_cube_map;
layout(set = 3, binding = 2) uniform textureCube irradiance_cube_map;
layout(set = 3, binding = 4) uniform texture2D spec_brdf_map;

#include "library/lighting.glsl"

const float MAX_SPEC_LOD = 4.0;

vec3 f_schlick(const vec3 f0, const float vh) {
	return f0 + (1.0 - f0) * exp2((-5.55473 * vh - 6.98316) * vh);
}

float v_smithschlick(const float nl, const float nv, const float a) {
	return 1.0 / ((nl * (1.0 - a) + a) * (nv * (1.0 - a) + a));
}

float d_ggx(const float nh, const float a) {
	float a2 = a * a;
	float denom = pow(nh * nh * (a2 - 1.0) + 1.0, 2.0);
	return a2 * (1.0 / 3.1415926535) / denom;
}

vec3 specularBRDF(const vec3 f0, const float roughness, const float nl, const float nh, const float nv, const float vh) {
	float a = roughness * roughness;
	return d_ggx(nh, a) * clamp(v_smithschlick(nl, nv, a), 0.0, 1.0) * f_schlick(f0, vh) / 4.0;
}

vec3 lambertDiffuseBRDF(const vec3 albedo, const float nl) {
	return albedo * max(0.0, nl);
}

vec3 saturate(vec3 v) {
    return clamp(v, vec3(0.0), vec3(1.0));
}

float saturate(float v) {
    return clamp(v, 0.0, 1.0);
}

void main() {
    vec4 main_color = texture(sampler2D(t_Color, s_Color), v_TexCoord);
    vec3 ambient = vec3(0.05, 0.05, 0.05);
    vec3 normal = normalize(i_normal);

    // vec3 ambient_irradiance = texture(samplerCube(irradiance_cube_map, v_TexCoord), N).rgb;
    // vec3 ambient_spec = textureLod(samplerCube(spec_cube_map, v_TexCoord), R, roughness * MAX_SPEC_LOD).rgb;
    // vec2 env_brdf = texture(sampler2D(spec_brdf_map, v_TexCoord), vec2(NdotV, roughness)).rg;

    // accumulate color
    vec3 color = ambient;
    for (int i=0; i < int(light_num.x) && i < MAX_LIGHTS; ++i) {
        DirectionalLight light = get_directional_light(i);
        float dot_product =  max(0.0, dot(normal, light.direction.xyz));
        color += dot_product * light.color.xyz;
    }

    // TODO: calculate attenuation.
    for (int i=0; i < int(light_num.y) && i < MAX_LIGHTS; ++i) {
        PointLight light = get_point_light(i);
        vec3 light_dir = normalize(light.position.xyz - i_position.xyz);
        float dot_product =  max(0.0, dot(normal, light_dir));
        color += dot_product * light.color.xyz;
    }

    outColor = vec4(color * main_color.xyz, main_color.w);
}