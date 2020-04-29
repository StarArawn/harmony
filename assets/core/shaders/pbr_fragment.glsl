#version 450
#extension GL_GOOGLE_include_directive : enable

layout(set = 1, binding = 0) uniform Globals {
    mat4 view_projection;
    vec4 camera_pos;
};

layout(location = 0) in vec2 i_uv;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec3 i_position;
layout(location = 3) in vec3 i_tangent;
layout(location = 4) in float i_tbn_handedness;
layout(location = 5) in mat3 i_TBN;
layout(location = 0) out vec4 outColor;

layout(set = 2, binding = 0) uniform Locals {
    vec4 color;
    // vec4 pbr_info;
};
layout(set = 2, binding = 1) uniform sampler tex_sampler;
layout(set = 2, binding = 2) uniform texture2D main_map;
layout(set = 2, binding = 3) uniform texture2D normal_map;
layout(set = 2, binding = 4) uniform texture2D metallic_roughness_map;

layout(set = 3, binding = 0) uniform textureCube irradiance_cube_map;
layout(set = 3, binding = 1) uniform textureCube spec_cube_map;
layout(set = 3, binding = 2) uniform texture2D spec_brdf_map;

#include "library/lighting.glsl"
#include "library/pbr.glsl"

vec4 toLinear(vec4 sRGB)
{
    bvec4 cutoff = lessThan(sRGB, vec4(0.04045));
    vec4 higher = pow((sRGB + vec4(0.055))/vec4(1.055), vec4(2.4));
    vec4 lower = sRGB/vec4(12.92);

    return mix(higher, lower, cutoff);
}

void main() {
    vec4 main_color = texture(sampler2D(main_map, tex_sampler), i_uv);
    
    vec3 normal = texture(sampler2D(normal_map, tex_sampler), i_uv).rgb;
    vec2 metallic_roughness = texture(sampler2D(metallic_roughness_map, tex_sampler), i_uv).bg;
    float metallic = metallic_roughness.x;
    float roughness = metallic_roughness.y;

    normal = (normal * 2.0 + 1.0) * 1.5;
    normal.xy *= vec2(1.0, 1.0);

    vec3 view = normalize(camera_pos.xyz - i_position.xyz);
    
    vec3 N = normalize(i_normal);

    // vec3 pos_dx = dFdx(i_position.xyz);
    // vec3 pos_dy = dFdy(i_position.xyz);
    // vec3 tex_dx = dFdx(vec3(i_uv, 0.0));
    // vec3 tex_dy = dFdy(vec3(i_uv, 0.0));
    // vec3 t = (tex_dy.t * pos_dx - tex_dx.t * pos_dy) / (tex_dx.s * tex_dy.t - tex_dy.s * tex_dx.t);
    // t = normalize(t - N * dot(N, t));
    
    //vec3 T = normalize(i_tangent); // - N * dot(N, i_tangent));
    // T = normalize(cross(B, N));

    vec3 T = i_tangent * vec3(-1.0, 1.0, 1.0);
    T = normalize(T - N * dot(N, T));
    vec3 B = cross(T, N) * i_tbn_handedness;
    // if(dot(cross(N.xyz, T.xyz), B.xyz) < 0.0) {
    //   T = T * -1;
    // }
    // B = cross(T, N) * i_tbn_handedness;
    mat3 TBN = mat3(T, B, N);
    N = i_TBN * normalize(normal);

    vec3 R = reflect(-view, N);
    float NdotV = abs(dot(N, view)) + 0.00001;

    vec3 f0 = mix(vec3(0.04), main_color.rgb, metallic);

    vec3 ambient_irradiance = texture(samplerCube(irradiance_cube_map, tex_sampler), N).rgb;
    vec3 ambient_spec = textureLod(samplerCube(spec_cube_map, tex_sampler), R, roughness * MAX_SPEC_LOD).rgb;
    vec2 env_brdf = texture(sampler2D(spec_brdf_map, tex_sampler), vec2(NdotV, roughness)).rg;

    vec3 ambient_spec_fres = f_schlick(f0, NdotV);
    vec3 ambient_diffuse_fac = vec3(1.0) - ambient_spec_fres;
    ambient_diffuse_fac *= 1.0 - metallic;

    vec3 ambient = (ambient_irradiance * main_color.rgb * ambient_diffuse_fac) + (ambient_spec * (ambient_spec_fres * env_brdf.x + env_brdf.y));
    float a = roughness * roughness;

    // // accumulate color
    vec3 light_acc = vec3(0.0);
    for (int i=0; i < int(light_num.x) && i < MAX_LIGHTS; ++i) {
        DirectionalLight light = get_directional_light(i);

        float d2 = dot(light.direction.xyz, light.direction.xyz);
        vec3 L = normalize(light.direction.xyz);
        vec3 H = normalize(view + L);
        vec3 l_contrib = light.color.xyz * 100.0 / d2;

        float NdotL = saturate(dot(N, L));
        float NdotH = saturate(dot(N, H));
        float VdotH = saturate(dot(H, view));
        vec3 fresnel = f_schlick(f0, VdotH);
        vec3 k_D = vec3(1.0) - fresnel;
        k_D *= 1.0 - metallic;

        vec3 specular = d_ggx(NdotH, a) * clamp(v_smithschlick(NdotL, NdotV, a), 0.0, 1.0) * fresnel;
        specular /= max(4.0 * NdotV * NdotL, 0.001);

        vec3 diffuse = main_color.rgb / 3.1415926535 * k_D;
        light_acc += (diffuse + specular) * NdotL * l_contrib;
    }

    // // TODO: calculate attenuation.
    // for (int i=0; i < int(light_num.y) && i < MAX_LIGHTS; ++i) {
    //     PointLight light = get_point_light(i);
    //     vec3 light_dir = normalize(light.position.xyz - i_position.xyz);
    //     float dot_product =  max(0.0, dot(normal, light_dir));
    //     light_acc += dot_product * light.color.xyz;
    // }

    //outColor = vec4(main_color.xyz * (0.5 * normalize(N) + vec3(1.0, 1.0, 1.0)), 1.0); //vec4(dot(normalize(N), vec3(0.0, 1.0, 0.5)).xxx, 1.0);
    //outColor = vec4(0.5 * (N + 1), 1.0); // * (max(dot(vec3(0.0, 1.0, 0.0), N), 0.0)), 1.0);
    // outColor = vec4(0.5 * (N + 1.0), 1.0);
    outColor = vec4(0.5 * (N + 1.0), 1.0); //(max(dot(vec3(0.5, 1.0, 0.5), N), 0.0)).xxx, 1.0);
    //outColor = vec4(i_uv.yy, 0.0, 1.0);
    outColor = vec4(dot(N, vec3(0.0, 1.0, 0.0)).xxx, 1.0);
    outColor = vec4(ambient_spec, 1.0);
}