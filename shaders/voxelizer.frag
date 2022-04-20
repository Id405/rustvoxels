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
layout(set = 2, binding = 1) buffer restrict coherent b_invoke_index {
    int index;
};

layout(set = 3, binding = 0) buffer writeonly b_invoke_positions {
    ivec4 invoke_positions[];
};

void main() {
    vec4 color = vec4(texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords).rgb, 1.0);
    ivec3 position = ivec3(gl_FragCoord.xzy * vec3(1.0, 512.0, 1.0));

    imageStore(u_voxel_grid, position, color);
    memoryBarrierBuffer();
    int fragment_index = atomicAdd(index, 1);
    invoke_positions[fragment_index] = ivec4(position, 1.0);
    
    f_color = vec4(1.0);
}