#version 450
#extension GL_GOOGLE_include_directive : enable

#include "library/lighting.glsl"
#include "library/pbr.glsl"
#include "library/common.glsl"

layout(set = 2, binding = 0) uniform Material {
    vec4 color;
    vec4 pbr_info;
};

layout(set = 2, binding = 1) uniform sampler tex_sampler;
layout(set = 2, binding = 2) uniform texture2D main_map;
layout(set = 2, binding = 3) uniform texture2D normal_map;
layout(set = 2, binding = 4) uniform texture2D metallic_roughness_map;

layout(set = 3, binding = 0) uniform textureCube irradiance_cube_map;
layout(set = 3, binding = 1) uniform textureCube spec_cube_map;
layout(set = 3, binding = 2) uniform texture2D spec_brdf_map;

layout(location = 0) in vec2 i_uv;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec3 i_position;
layout(location = 3) in vec3 i_tangent;
layout(location = 4) in float i_tbn_handedness;
layout(location = 0) out vec4 outColor;

void main() {
    vec4 main_color = texture(sampler2D(main_map, tex_sampler), i_uv);
    
    vec2 metallic_roughness = texture(sampler2D(metallic_roughness_map, tex_sampler), i_uv).bg;
    float metallic = metallic_roughness.x * pbr_info.x;
    float roughness = metallic_roughness.y * pbr_info.y;
    
    vec3 normal = texture(sampler2D(normal_map, tex_sampler), i_uv).rgb;
    normal = normal * 2.0 - 1.0;
    vec3 V = normalize(camera_pos.xyz - i_position.xyz);
    vec3 N = normalize(i_normal);
    vec3 T = normalize(i_tangent);
    vec3 B = cross(N, T) * i_tbn_handedness;
    mat3 TBN = mat3(T, B, N);
    N = TBN * normalize(normal);
    
    float NdotV = abs(dot(N, V)) + 0.00001;
    vec3 R = reflect(-V, N);

    vec3 ambient_irradiance = texture(samplerCube(irradiance_cube_map, tex_sampler), N).rgb;
    vec3 ambient_spec = textureLod(samplerCube(spec_cube_map, tex_sampler), R, roughness * MAX_SPEC_LOD).rgb;
    vec2 env_brdf = texture(sampler2D(spec_brdf_map, tex_sampler), vec2(NdotV, roughness)).rg;

    vec3 f0 = mix(vec3(0.04), main_color.xyz, metallic);

    vec3 ambient_spec_fres = f_schlick(f0, NdotV);

    vec3 ambient_diffuse_fac = vec3(1.0) - ambient_spec_fres;
    ambient_diffuse_fac *= 1.0 - metallic;

    vec3 ambient = (ambient_irradiance * main_color.xyz * ambient_diffuse_fac) + (ambient_spec * (ambient_spec_fres * env_brdf.x + env_brdf.y));

    float a = roughness * roughness;

    vec3 light_acc = vec3(0.0);
    for (int i=0; i < int(light_num.x) && i < MAX_LIGHTS; ++i) {
        DirectionalLight light = get_directional_light(i);

        float d2 = dot(light.direction.xyz, light.direction.xyz);
        vec3 L = normalize(light.direction.xyz);
        vec3 H = normalize(V + L);
        vec3 l_contrib = light.color.xyz * 5.0 / d2;

        float NdotL = saturate(dot(N, L));
        float NdotH = saturate(dot(N, H));
        float VdotH = saturate(dot(H, V));
        vec3 fresnel = f_schlick(f0, VdotH);
        vec3 k_D = vec3(1.0) - fresnel;
        k_D *= 1.0 - metallic;

        vec3 specular = d_ggx(NdotH, a) * clamp(v_smithschlick(NdotL, NdotV, a), 0.0, 1.0) * fresnel;
        specular /= max(4.0 * NdotV * NdotL, 0.001);

        vec3 diffuse = main_color.rgb / 3.1415926535 * k_D;
        light_acc += (diffuse + specular) * NdotL * l_contrib;
    }


    outColor = vec4(ambient + light_acc, 1.0);
}