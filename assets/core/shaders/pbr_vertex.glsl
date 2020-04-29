#version 450

layout(location = 0) in vec3 i_Pos;
layout(location = 1) in vec3 i_normal;
layout(location = 2) in vec2 i_uv;
layout(location = 3) in vec4 i_tangent;
layout(location = 0) out vec2 v_TexCoord;
layout(location = 1) out vec3 o_normal;
layout(location = 2) out vec3 o_position;
layout(location = 3) out vec3 o_tangent;
layout(location = 4) out float o_tbn_handedness;
layout(location = 5) out mat3 o_TBN;

layout(set = 1, binding = 0) uniform Globals {
    mat4 view_projection;
    mat4 camera_pos;
    mat4 view;
};

layout(set = 0, binding = 0) uniform Locals {
    mat4 world;
};

mat4 rotationZ( in float angle ) {
	return mat4(	cos(angle),		-sin(angle),	0,	0,
			 		sin(angle),		cos(angle),		0,	0,
							0,				0,		1,	0,
							0,				0,		0,	1);
}

mat4 rotationX( in float angle ) {
	return mat4(	1.0,		0,			0,			0,
			 		0, 	cos(angle),	-sin(angle),		0,
					0, 	sin(angle),	 cos(angle),		0,
					0, 			0,			  0, 		1);
}

void main() {
    v_TexCoord = vec2(i_uv.x, i_uv.y);
    mat3 normalMatrix = mat3(inverse(world));
    // o_normal = normalize(vec3(i_normal)); //(world * vec4(i_normal, 0.0)).xyz;
    o_position = (world * vec4(i_Pos, 1.0)).xyz;
    // o_tangent = normalize(vec3(i_tangent.xyz)); //normalize(world * vec4(i_tangent.xyz, 0.0)).xyz;
    // vec3 bitangentW = cross(o_normal, o_tangent.xyz) * i_tangent.w;
    // o_TBN = mat3(o_tangent, bitangentW, o_normal);
    o_tbn_handedness = i_tangent.w;

    //vec3 ta = i_tangent.xyz * vec3(1.0, 1.0, 1.0);
    //ta = (vec4(ta, 1.0) * rotationZ(3.145)).xyz;
    o_normal = normalize(normalMatrix * i_normal.xyz);
    o_tangent = normalize(normalMatrix * i_tangent.xyz);
    // t = normalize(t - n * dot(n, t));
    // vec3 b = normalize(cross( n, t ) * i_tangent.w);
    // o_TBN = transpose(mat3(t, b, n));

    gl_Position = view_projection * world * vec4(i_Pos, 1.0);
}