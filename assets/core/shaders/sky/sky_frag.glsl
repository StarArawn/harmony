#version 450

#include "./color.glsl"

// Normalization factors from the precomputation phase.
const vec2 RAYLEIGH_NORM = vec2(0.0, 0.05588319);
const vec2 MIE_NORM = vec2(0.0, .02527083);

// Spectral irradiance and spectral to RGB conversion constants from
// the precomputation phase.
const vec3 SPECTRAL_IRRADIANCE = vec3(1.526, 1.91, 2.08) / 10.0;
const vec3 SPECTRAL_TO_RGB = vec3(133.3209, 88.51855, 112.7552);

const float SUN_ANGULAR_RADIUS = 0.004675034;

// TODO: figure out how to use different bind sets here..
// START LIGHTING
const int MAX_LIGHTS = 10;

struct DirectionalLight {
    vec4 direction;
    vec4 color;
};

struct PointLight {
    vec4 position;
    vec4 color;
    vec4 attenuation;
};

layout (set = 0, binding = 1) uniform LightingData {
    vec4 light_num;
    DirectionalLight directional_lights[MAX_LIGHTS / 2];
    PointLight point_lights[MAX_LIGHTS / 2];
};

DirectionalLight get_directional_light(int index) {
    return directional_lights[index];
}

PointLight get_point_light(int index) {
    return point_lights[index];
}
// END LIGHTING

layout(location = 0) in vec3 v_Uv;
layout(location = 0) out vec4 f_Color;

layout(set = 0, binding = 0) uniform Globals {
    mat4 view_projection;
    vec4 camera_pos;
    mat4 view;
    mat4 projection;
};

layout(set = 1, binding = 0) uniform sampler tex_sampler;
layout(set = 1, binding = 1) uniform texture2D rayleighTexture;
layout(set = 1, binding = 2) uniform texture2D mieTexture;

const float exposure = 2;
const float mieG = 0.99;

float RayleighPhaseFunction(float cosTheta)
{
    // Original rayleigh phase function.
    // return 0.75 * (1 + pow(cosTheta, 2));
    
    // Modified to better account for sun-view azimuth as described in Section 4.1 of:
    // http://publications.lib.chalmers.se/records/fulltext/203057/203057.pdf
    return 0.8 * (1.4 + 0.5*cosTheta);
}

float MiePhaseFunction(float cosTheta, float g)
{
    float g2 = g * g;
    float t2 = cosTheta * cosTheta;
    float result = 3.0 / 2.0;
    result *= (1.0 - g2) / (2.0 + g2);
    result *= (1.0 + t2) / pow(1.0 + g2 - 2.0*g*t2, 3.0/2.0);
    return result;
}

vec3 uncharted2Tonemap(vec3 x) {
  float A = 0.15;
  float B = 0.50;
  float C = 0.10;
  float D = 0.20;
  float E = 0.02;
  float F = 0.30;
  float W = 11.2;
  return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

vec3 uncharted2(vec3 color) {
  const float W = 11.2;
  float exposureBias = 2.0;
  vec3 curr = uncharted2Tonemap(exposureBias * color);
  vec3 whiteScale = 1.0 / uncharted2Tonemap(vec3(W));
  return curr * whiteScale;
}

mat3 QuaternionToMatrix(vec4 quat)
{
    vec3 cross = quat.yzx * quat.zxy;
    vec3 square= quat.xyz * quat.xyz;
    vec3 wimag = quat.w * quat.xyz;

    square = square.xyz + square.yzx;

    vec3 diag = 0.5 - square;
    vec3 a = (cross + wimag);
    vec3 b = (cross - wimag);

    return mat3(
    2.0 * vec3(diag.x, b.z, a.y),
    2.0 * vec3(a.z, diag.y, b.x),
    2.0 * vec3(b.y, a.x, diag.z));
}

vec3 rescaleHDR(vec3 hdrPixel)
{
    vec4 IBLCorrection = vec4(0.0, 1.0, 0.0, 0.0);
    vec4 IblMaxValue = vec4(0.0, 0.0, 0.0, 1.0);
   if (hdrPixel.x < 0)
    hdrPixel.x = 0;
   if (hdrPixel.y < 0)
    hdrPixel.y = 0;
   if (hdrPixel.z < 0)
    hdrPixel.z = 0;

   float intensity  = float(dot(hdrPixel, vec3(0.299f,0.587f,0.114f)));

   if (intensity > 1)
   {
       hdrPixel = hdrPixel - IBLCorrection.x * (hdrPixel - IblMaxValue.rgb) * hdrPixel * (hdrPixel - (IblMaxValue.rgb * 0.5));
   }
   else
   {
       hdrPixel = hdrPixel - IBLCorrection.x * (hdrPixel - IblMaxValue.rgb) * hdrPixel * (hdrPixel - 0.5);
   }

   // Saturation adjustment
   hdrPixel = mix(intensity.xxx, hdrPixel, IBLCorrection.y);

   // Hue adjustment      
   const vec3 root = vec3(0.57735, 0.57735, 0.57735);
   float half_angle = 0.5 * radians(IBLCorrection.z); // Hue is radians of 0 tp 360 degree
   vec4 rot_quat = vec4( (root * sin(half_angle)), cos(half_angle));
   mat3 rot_Matrix = QuaternionToMatrix(rot_quat);     
   hdrPixel = rot_Matrix * hdrPixel;
   hdrPixel = hdrPixel * exposure;

   return hdrPixel; 
}

void main() {
    // Sunset
    vec3 sun_pos = vec3(0, -0.01, 1);
    // Mid-day
    // vec3 sun_pos = vec3(0, 1, 0);
    if (int(light_num.x) > 0) {
        DirectionalLight light = directional_lights[0];
        sun_pos = light.direction.xyz;
    }

    mat3 invModelView = transpose(mat3(view));
    vec3 unProjected = (inverse(projection) * vec4(v_Uv, 1.0)).xyz;
    vec3 view_dir = invModelView * unProjected;

    vec3 viewDir = normalize(view_dir);
    vec3 lightDir = normalize(sun_pos);

    // Calculate the view-zenith and sun-zenith angles.
    float cosV = dot(viewDir, vec3(0, -1, 0));
    float cosL = dot(lightDir, vec3(0, 1, 0));

    // Convert the angles to texture coordinates using the parameterization function.
    // Note: we use abs+sign to avoid negative roots!
    float u = 0.5 * (1.0 + sign(cosV)*pow(abs(cosV), 1.0/3.0));
    float v = 0.5 * (1.0 + sign(cosL)*pow(abs(cosL), 1.0/3.0));

    // Sample the textures.
    vec3 rayleigh = texture(sampler2D(rayleighTexture, tex_sampler), vec2(u, 1.0 - v)).rgb;
    vec3 mie = texture(sampler2D(mieTexture, tex_sampler), vec2(u, 1.0 - v)).rgb;

    // Remap the values.
    rayleigh = rayleigh*(RAYLEIGH_NORM.y-RAYLEIGH_NORM.x) + RAYLEIGH_NORM.x;
    mie = mie*(MIE_NORM.y-MIE_NORM.x) + MIE_NORM.x;

    // Calculate the view-sun angle for the phase function.
    // Note: we clamp it between [0, 1] or else we would get the sun
    // on both sides of the light direction.
    float cosTheta = dot(viewDir, lightDir);
    cosTheta = clamp(cosTheta, 0, 1);

    // Apply the phase function.
    rayleigh *= RayleighPhaseFunction(cosTheta);
    mie *= MiePhaseFunction(cosTheta, mieG);

    // Compute the scattering, and apply the spectral irradiance to
    // get the spectral radiance for this fragment.
    vec3 radiance = vec3(0.0);
    radiance += rayleigh;
    radiance += mie;
    radiance *= SPECTRAL_IRRADIANCE; // * vec3(exposure);

    // Multiply by the SPECTRAL_TO_RGB conversion constants to convert
    // the spectral radiance to RGB values.
    vec3 rgb = radiance * SPECTRAL_TO_RGB;

    if (acos(cosTheta) < SUN_ANGULAR_RADIUS)
    {
        // TODO: this is not physically correct. It only works for exposure < 1.
        // Technically it should be multiplied by the transmittance.
        rgb /= SPECTRAL_IRRADIANCE; // * vec3(exposure);
    }

    // Tonemap the resulting RGB samples into a valid RGB range.
    // rgb = pow(vec3(1.0) - exp(-rgb), vec3(1.0/2.2));
    // rgb = 1.0 - exp(-rgb);
    //rgb = uncharted2(rgb);

    rgb = rescaleHDR(rgb);

    f_Color = vec4(rgb, 1);
}
