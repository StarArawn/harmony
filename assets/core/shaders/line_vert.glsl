#version 450

layout(location = 0) in vec3 i_Pos;
layout(location = 1) in vec3 i_color;
layout(location = 0) out vec3 o_color;

layout(set = 0, binding = 0) uniform Globals {
    mat4 view_projection;
};

void main() {
    o_color = i_color;
    gl_Position = view_projection * vec4(i_Pos, 1.0);
}