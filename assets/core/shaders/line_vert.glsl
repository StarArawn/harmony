#version 450

layout(location = 0) in vec3 i_Pos;

layout(set = 0, binding = 0) uniform Globals {
    mat4 view_projection;
};

void main() {
    gl_Position = view_projection * vec4(i_Pos, 1.0);
}