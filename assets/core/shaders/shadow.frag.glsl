#version 450

layout(location = 0) in vec4 i_position;

layout (location = 0) out float outColor;

layout(push_constant) uniform Uniforms {
    mat4 view_proj;
    vec4 light_pos;
};

void main() {
    // vec3 light_vec = i_position.xyz - light_pos.xyz;
    // outColor = 0; //length(light_vec) / 1000.0;
}
