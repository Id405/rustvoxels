#version 460
#extension GL_EXT_scalar_block_layout : require

layout (location = 0) out vec4 outColor;
layout (location = 1) out vec4 outDepth;

// Do we really need all these samplers? I'm not so sure
layout (set=0, binding = 0) uniform texture2D rendered_frame_texture;
layout (set=0, binding = 1) uniform sampler rendered_frame_sampler;
layout (set=0, binding = 2) uniform texture2D past_frame_texture;
layout (set=0, binding = 3) uniform sampler past_frame_sampler;
layout (set=0, binding = 4) uniform texture2D rendered_frame_depth_texture;
layout (set=0, binding = 5) uniform sampler rendered_frame_depth_sampler; 
layout (set=0, binding = 5) uniform texture2D past_frame_depth_texture;
layout (set=0, binding = 6) uniform sampler past_frame_depth_sampler; 

layout (set=1, binding=0, std430) uniform Uniforms {
    mat4 invPastCameraMatrix;
    mat4 camera_matrix;
    ivec2 resolution; 
    float focal_length;
    int frame_count;
};

#define repro_percent 0.90

void main() {
    // outColor = texture(sampler2D(rendered_frame_texture, rendered_frame_sampler), gl_FragCoord.xy/vec2(resolution)) * vec4(1.0 - 0.90)
    //  + texture(sampler2D(past_frame_texture, past_frame_sampler), gl_FragCoord.xy/vec2(resolution)) * vec4(0.90);
    // outColor = texture(sampler2D(rendered_frame_depth_texture, rendered_frame_depth_sampler), gl_FragCoord.xy/vec2(resolution));
    // outColor = vec4(vec3(vec2(gl_FragCoord.xy/vec2(resolution)), 0.0), 1.0);

    vec2 resolution = vec2(resolution.x, resolution.y);

    vec2 textureCoordinate = gl_FragCoord.xy/resolution;
    float reprojectionPercentWeighted = repro_percent;

    // Get the freshly rendered color and depth information
    vec4 renderedFrameColor = texture(sampler2D(rendered_frame_texture, rendered_frame_sampler), gl_FragCoord.xy/vec2(resolution));
    float renderedFrameDepthFloat = texture(sampler2D(rendered_frame_depth_texture, rendered_frame_depth_sampler), gl_FragCoord.xy/vec2(resolution)).r*10000;

    // Setup a raycast to find the worldspace position of the current pixel
    vec2 s = vec2((gl_FragCoord.x) - resolution.x/2.0f, (resolution.y - gl_FragCoord.y) - resolution.y/2.0f);
	vec3 raypos = (camera_matrix * vec4(0, 0, 0, 1)).xyz; //TODO precompute these values
	vec3 raydir = normalize(vec3(s.x/resolution.y, focal_length, s.y/resolution.y));
	raydir = (camera_matrix * vec4(raydir, 0.0)).xyz;
    vec3 worldSpacePosition = raypos + raydir * renderedFrameDepthFloat;

    // Then transform that world space position into a camera space position for the last frame
    vec3 cameraSpacePosition = (invPastCameraMatrix * vec4(worldSpacePosition, 1.0)).xyz;

    // Project the world space position into camera space
    vec2 prevUV = cameraSpacePosition.xz/(cameraSpacePosition.y/focal_length);
    prevUV.x /= resolution.x/resolution.y;
    prevUV += 0.5;
    prevUV.y = 1.0 - prevUV.y;

    // Then get the color of that pixel
    // vec4 pastFrameColor = texelFetch(pastFrame, ivec2(prevUV * resolution), 0);
    vec4 pastFrameColor = texture(sampler2D(past_frame_texture, past_frame_sampler), prevUV);
    float pastFrameDepth = texture(sampler2D(past_frame_depth_texture, past_frame_depth_sampler), prevUV).a*10000;
    // pastFrameColor.rgb = pow(pastFrameColor.rgb, vec3(2.2)); // Reverse the srgb color transform applied to it

    // If the camera space coordinate is outside of the previous frame then reject it.
    if (cameraSpacePosition.y < focal_length || any(greaterThan(prevUV, vec2(1))) || any(lessThan(prevUV, vec2(0)))) {
        reprojectionPercentWeighted = 0;
    }

    // Don't reproject the sky
    if(renderedFrameDepthFloat == 0.0) {
        reprojectionPercentWeighted = 0.0;
        pastFrameDepth = 0.0;
    }

    if(pastFrameDepth == 0.0) {
        reprojectionPercentWeighted = 0.0;
    }

    // Finally average out the depth and color information
    outColor = renderedFrameColor * (1.0 - reprojectionPercentWeighted) + pastFrameColor * reprojectionPercentWeighted;
    outDepth = vec4(renderedFrameDepthFloat/10000);
    // outColor = vec4(pastFrameDepth/10000);

    // Uncomment this code to render once a second and extrapolate between frames
    // if(frameCount % 60 == 0) {
    //     outColor = renderedFrameColor;
    //     gl_FragDepth = renderedFrameDepthFloat/10000;
    // } else {
    //     outColor = pastFrameColor;
    //     if(pastFrameColor.a < 0.1 || reprojectionPercentWeighted < 0.1) {
    //         outColor = vec4(1.0, 0.0, 0.0, 1.0);
    //     }
    // }

    // And apply an srgb color transform
    // outColor = pow(outColor, vec4(vec3(1.0/2.2), 1.0));

    // outColor = vec4(vec3(prevUV, 0.0), 1.0);
    outColor = vec4(vec3(ivec3(floor(worldSpacePosition)) % ivec3(2.0)), 1.0);
    // outColor = vec4(vec3(abs(minDepthDistance)), 1.0);
    // outColor = vec4(vec3(length(abs(prevUV-textureCoordinate)) * 20), 1.0);
    // outColor = vec4(vec3(prevUV, 0.0), 1.0);
}