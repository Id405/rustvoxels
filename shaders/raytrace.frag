#version 450

layout(location=0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform Uniforms {
    uvec2 resolution;
};

void main() {
    frag_color = vec4(vec3(gl_FragCoord.xy/(vec2(resolution)), 0.0), 1.0);
}