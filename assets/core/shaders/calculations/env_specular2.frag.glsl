//------------------------------------------------------------------------------------//
//                                                                                    //
//    ._____________.____   __________         __                                     //
//    |   \______   \    |  \______   \_____  |  | __ ___________                     //
//    |   ||    |  _/    |   |    |  _/\__  \ |  |/ // __ \_  __ \                    //
//    |   ||    |   \    |___|    |   \ / __ \|    <\  ___/|  | \/                    //
//    |___||______  /_______ \______  /(____  /__|_ \\___  >__|                       //
//                \/        \/      \/      \/     \/    \/                           //
//                                                                                    //
//    IBLBaker is provided under the MIT License(MIT)                                 //
//    IBLBaker uses portions of other open source software.                           //
//    Please review the LICENSE file for further details.                             //
//                                                                                    //
//    Copyright(c) 2014 Matt Davidson                                                 //
//                                                                                    //
//    Permission is hereby granted, free of charge, to any person obtaining a copy    //
//    of this software and associated documentation files(the "Software"), to deal    //
//    in the Software without restriction, including without limitation the rights    //
//    to use, copy, modify, merge, publish, distribute, sublicense, and / or sell     //
//    copies of the Software, and to permit persons to whom the Software is           //
//    furnished to do so, subject to the following conditions :                       //
//                                                                                    //
//    1. Redistributions of source code must retain the above copyright notice,       //
//    this list of conditions and the following disclaimer.                           //
//    2. Redistributions in binary form must reproduce the above copyright notice,    //
//    this list of conditions and the following disclaimer in the                     //
//    documentation and / or other materials provided with the distribution.          //
//    3. Neither the name of the copyright holder nor the names of its                //
//    contributors may be used to endorse or promote products derived                 //
//    from this software without specific prior written permission.                   //
//                                                                                    //
//    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR      //
//    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,        //
//    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.IN NO EVENT SHALL THE      //
//    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER          //
//    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,   //
//    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN       //
//    THE SOFTWARE.                                                                   //
//                                                                                    //
//------------------------------------------------------------------------------------//

#version 450
#extension GL_ARB_separate_shader_objects : enable

#include "brdf.glsl"

layout(location = 0) in vec3 f_pos;
layout(location = 1) flat in int face_index;

layout(set = 0, binding = 0) uniform Globals {
    // (ConvolutionSamplesOffset, ConvolutionSampleCount, ConvolutionMaxSamples, env scale)
    vec4 data;
    // (width, height, ConvolutionRoughness, ConvolutionMip)
    vec4 data2;
};

layout(set = 0, binding = 1) uniform textureCube env_texture;
layout(set = 0, binding = 2) uniform textureCube last_result;
layout(set = 0, binding = 3) uniform sampler env_sampler;

layout(location = 0) out vec4 color;

const vec4 IBLCorrection = vec4(0.0, 1.0, 0.0, 0.0);
const vec4 IblMaxValue = vec4(0.0, 0.0, 0.0, 1.0);

//
// Attributed to:
// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
// Holger Dammertz.
// 
vec2 Hammersley(uint i, uint N) 
{
    float ri = bitfieldReverse(i) * 2.3283064365386963e-10f;
    return vec2(float(i) / float(N), ri);
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
   hdrPixel = hdrPixel * data.w;

   return hdrPixel; 
}

vec3 importance_sample(vec3 N)
{
    vec3 V = N;
    vec4 result = vec4(0,0,0,0);
    float SampleStep = (data.z / data.y);
    uint sampleId = uint(data.x);

    for(uint i = 0; i < uint(data.y); i++ )
    {

        vec2 Xi = Hammersley(sampleId, uint(data.z));
        vec3 H = importanceSampleGGX( Xi, data2.z, N);
        vec3 L = 2 * dot( V, H ) * H - V;
        float NoL = max(dot( N, L ), 0);
        //float VoL = max (dot(V, L), 0);
        float NoH = max(dot( N, H ), 0);
        float VoH = max(dot( V, H ), 0);
        if (NoL > 0.0)
        {
            //
            // Compute pdf of BRDF
            // Taken from Epic's Siggraph 2013 Lecture:
            // http://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
            //
            float Dh = specularD(data2.z, NoH);
            float pdf = Dh * NoH / (4*VoH);
            float solidAngleTexel = 4 * PI / (6 * data2.y * data2.x);
            float solidAngleSample = 1.0 / (data.y * pdf);
            float lod = uint(data2.z) == 0 ? 0 : 0.5 * log2(float(solidAngleSample/solidAngleTexel));

            vec3 specSample = rescaleHDR(texture(samplerCube(env_texture, env_sampler), L, lod).rgb);
            // vec3 diffuseSample = texture(samplerCube(env_texture, env_sampler), H).rgb;
            result = sumSpecular(specSample, NoL, result);
        }
        sampleId += uint(SampleStep);
   }
   if (result.w == 0)
        return result.xyz;
   else   
       return (result.xyz / result.w);
}

void main() {
    vec3 pos = f_pos;
    vec3 N = normalize(pos);
    vec4 sampledColor = vec4(0,0,0,1);

    // Sample source cubemap at specified mip.
    vec3 importanceSampled = importance_sample(N);

    if (data.x > 1e-6)
    {
        vec3 lastResult = texture(samplerCube(last_result, env_sampler), N, int(data2.w)).rgb;
        sampledColor.rgb = mix(lastResult.xyz, importanceSampled.xyz, 1.0 / data.x);
    }
    else 
    {
        sampledColor.xyz = importanceSampled.xyz;
    }
    color = sampledColor;
}