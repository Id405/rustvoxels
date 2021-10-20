#version 460
#extension GL_EXT_scalar_block_layout : require

layout (location=0) out vec4 outColor;

layout(set=1, binding=0, std430) uniform Denoise {
    ivec2 resolution;
};

layout(set=0, binding=0) uniform texture2D raw_texture;
layout(set=0, binding=1) uniform sampler raw_sampler;

void main() {
    outColor = texture(sampler2D(raw_texture, raw_sampler), gl_FragCoord.xy/vec2(resolution));
    // outColor = vec4(vec3(vec2(gl_FragCoord.xy/vec2(resolution)), 0.0), 1.0);
}