#version 450
#extension GL_GOOGLE_include_directive : enable

#include "library/lighting.glsl"
#include "library/pbr.glsl"
#include "library/common.glsl"

layout(set = 2, binding = 0) uniform Material {
    vec4 color;
    // (metallic, roughness, metallic_amount, roughness_amount)
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

const float roughnessRescale = 1.0;
const float specularIntensity = 0.1;

const float PI = 3.14159265358979323;

// case RoughnessMetal:
// {
// specularModifiers.x = 0.0;
// specularModifiers.y = 0.0;
// break;
// }
// case GlossMetal:
// {
// specularModifiers.x = 1.0;
// specularModifiers.y = 0.0;
// break;
// }
// case RoughnessInverseMetal:
// {
// specularModifiers.x = 0.0;
// specularModifiers.y = 1.0;
// break;
// }
// case GlossInverseMetal:
// {
// specularModifiers.x  = 1.0;
// specularModifiers.y = 1.0;
// break;
// }

vec3
shade (float vdotn, 
       float roughness, 
       float metalness, 
       vec3 diffuseColor,
       vec3 diffuseIBL, 
       vec3 specularIBL,
       vec3 normal)
{
    roughness = roughness * (1.0 / roughnessRescale);

    vec2 brdfTerm = texture(sampler2D(spec_brdf_map, tex_sampler), vec2(vdotn, 1.0 - roughness)).rg;
    vec3 metalSpecularIBL = specularIBL.rgb; 

    vec3 dielectricColor = vec3(0.04, 0.04, 0.04);
    vec3 diffColor = diffuseColor.rgb * (1.0 - metalness);
    vec3 specColor = mix(dielectricColor.rgb, diffuseColor.rgb, metalness) * specularIntensity;
   
    // TODO: Add AO.
    // diffuseIBL.rgb = lerp(diffuseIBL.rgb * 0.3f, diffuseIBL.rgb, bakedAO);
    
    vec3 albedoByDiffuse = diffColor.rgb * diffuseIBL.rgb;

    vec3 litColor =  (albedoByDiffuse.rgb + (metalSpecularIBL * (specColor * brdfTerm.x + (brdfTerm.y)))); // * bakedAO;
    return litColor;
} 

vec3 Uncharted2ToneMapping(vec3 color)
{
	float A = 0.15;
	float B = 0.50;
	float C = 0.10;
	float D = 0.20;
	float E = 0.02;
	float F = 0.30;
	float W = 11.2;
	float exposure = 2.;
	color *= exposure;
	color = ((color * (A * color + C * B) + D * E) / (color * (A * color + B) + D * F)) - E / F;
	float white = ((W * (A * W + C * B) + D * E) / (W * (A * W + B) + D * F)) - E / F;
	color /= white;
	color = pow(color, vec3(1. / 1.2));
	return color;
}

float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;
	
    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
	
    return num / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
	
    return num / denom;
}
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);
	
    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

void main() {
    vec3 main_color = texture(sampler2D(main_map, tex_sampler), i_uv).rgb * color.rgb;
    
    vec2 metallic_roughness = texture(sampler2D(metallic_roughness_map, tex_sampler), i_uv).bg;
    float metallic = mix(metallic_roughness.x, pbr_info.x, pbr_info.z);
    float roughness = mix(metallic_roughness.y, pbr_info.y, pbr_info.w);
    
    vec3 normal = texture(sampler2D(normal_map, tex_sampler), i_uv).rgb;
    normal = normal * 2.0 - 1.0;
    vec3 V = normalize(camera_pos.xyz - i_position.xyz);
    vec3 N = normalize(i_normal);
    vec3 T = normalize(i_tangent);
    vec3 B = cross(N, T) * i_tbn_handedness;
    mat3 TBN = mat3(T, B, N);
    N = TBN * normalize(normal);
    
    float VdotN = dot(V, N);
    vec3 R = reflect(-V, N);

    vec3 ambient_irradiance = texture(samplerCube(irradiance_cube_map, tex_sampler), N).rgb;
    vec3 ambient_spec = textureLod(samplerCube(spec_cube_map, tex_sampler), R, roughness * MAX_SPEC_LOD).rgb;
    // vec2 env_brdf = texture(sampler2D(spec_brdf_map, tex_sampler), vec2(NdotV, roughness)).rg;

    // vec3 f0 = mix(vec3(0.04), main_color.xyz, metallic);

    // vec3 ambient_spec_fres = f_schlick(f0, NdotV);

    // vec3 ambient_diffuse_fac = vec3(1.0) - ambient_spec_fres;
    // ambient_diffuse_fac *= 1.0 - metallic;

    // vec3 ambient = (ambient_irradiance * main_color.xyz * ambient_diffuse_fac) + (ambient_spec * (ambient_spec_fres * env_brdf.x + env_brdf.y));

    float a = roughness * roughness;

    // Convert irradiance to radiance
    ambient_irradiance = (ambient_irradiance / 3.145) * 1.0; // 1.0 is enviroment scale

    roughness = mix(roughness, 1.0 - roughness, 0.0);
    metallic = mix(metallic, 1.0 - metallic, 0.0);

    // TODO: Point-lights?
    vec3 ambient = shade(VdotN, roughness, metallic, main_color.rgb, ambient_irradiance, ambient_spec, N);

    vec3 F0 = vec3(0.04); 
    F0 = mix(F0, main_color, metallic);

    // Directional Lighting
    vec3 light_acc = vec3(0.0);
    for (int i=0; i < int(light_num.x) && i < MAX_LIGHTS; ++i) {

        DirectionalLight light = directional_lights[i];
        // calculate per-light radiance
        vec3 L = normalize(light.direction.xyz);
        vec3 H = normalize(V + L);
        vec3 radiance = light.color.xyz * 10;        
        
        // cook-torrance brdf
        float NDF = DistributionGGX(N, H, roughness);        
        float G   = GeometrySmith(N, V, L, roughness);      
        vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);       
        
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;	  
        
        vec3 numerator    = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
        vec3 specular     = numerator / max(denominator, 0.001);  
            
        // add to outgoing radiance Lo
        float NdotL = max(dot(N, L), 0.0);                
        light_acc += (kD * main_color / PI + specular) * radiance * NdotL; 
    }

    vec3 color = Uncharted2ToneMapping(ambient + light_acc);

    outColor = vec4(color, 1.0);
}