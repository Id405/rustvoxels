#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;

layout(location=0) out vec2 v_tex_coords;

layout(set=0, binding=0) uniform Camera {
    mat4 u_view_proj;
    mat4 u_model;
};

void main() {
    v_tex_coords = vec2(a_tex_coords.x, 1.0 - a_tex_coords.y);
    gl_Position = u_view_proj * u_model * vec4(a_position, 1.0);
    // gl_Position = vec4(a_position, 1.0);
}