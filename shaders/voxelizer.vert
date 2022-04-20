#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec2 v_tex_coords;

layout(set=0, binding=0) uniform Camera {
    uint scene_size;
    mat4 u_model;
};

void main() {
    v_tex_coords = (vec4(a_position, 1.0)).xy;
    gl_Position = u_model * vec4(a_position, 1.0);
    // gl_Position = vec4(a_position, 1.0);
}