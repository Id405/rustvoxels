#version 460
#extension GL_EXT_scalar_block_layout : require

layout (location = 0) out vec4 outColor;

layout (set=0, binding = 0) uniform texture2D rendered_frame_texture;
layout (set=0, binding = 1) uniform sampler rendered_frame_sampler;
layout (set=0, binding = 2) uniform texture2D past_frame_texture;
layout (set=0, binding = 3) uniform sampler past_frame_sampler;

layout (set=1, binding=0, std430) uniform Uniforms {
    mat4 invPastCameraMatrix;
    mat4 cameraMatrix;
    ivec2 resolution; 
    float focal_length;
    int frame_count;
};

void main() {
    outColor = texture(sampler2D(rendered_frame_texture, rendered_frame_sampler), gl_FragCoord.xy/vec2(resolution)) * vec4(1.0 - 0.90)
     + texture(sampler2D(past_frame_texture, past_frame_sampler), gl_FragCoord.xy/vec2(resolution)) * vec4(0.90);
    // outColor = texture(sampler2D(rendered_frame_texture, rendered_frame_sampler), gl_FragCoord.xy/vec2(resolution));
    // outColor = vec4(vec3(vec2(gl_FragCoord.xy/vec2(resolution)), 0.0), 1.0);
}