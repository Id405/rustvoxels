#version 450

layout(location=0) in vec2 v_tex_coords;

layout(location=0) out vec4 f_color;

layout(set=0, binding=0) uniform Camera {
    uint scene_size;
    mat4 u_model;
};

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

layout(rgba32f, set = 2, binding = 0) uniform writeonly restrict image3D u_voxel_grid;

void main() {
    vec4 color = vec4(texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords).rgb, 1.0);
    ivec3 position = ivec3(gl_FragCoord.xzy * vec3(1.0, 512.0, 1.0));

    imageStore(u_voxel_grid, position, color);
    
    f_color = vec4(1.0);
}