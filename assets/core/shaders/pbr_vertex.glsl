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
    mat3 normalMatrix = mat3(transpose(inverse(world)));
    o_position = (world * vec4(i_Pos, 1.0)).xyz;
    o_normal = normalize(world * vec4(i_normal.xyz, 0.0)).xyz;
    o_tangent = normalize(world * vec4(i_tangent.xyz, 0.0)).xyz;
    o_tbn_handedness = i_tangent.w;
    gl_Position = view_projection * world * vec4(i_Pos, 1.0);
}