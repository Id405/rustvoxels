#version 460
#extension GL_EXT_scalar_block_layout : require

layout (location = 0) out vec4 outColor;
layout (location = 1) out vec4 outDepth;

// TODO Do we really need all these samplers? I'm not so sure
layout (set=0, binding = 0) uniform texture2D rendered_frame_texture;
layout (set=0, binding = 1) uniform sampler rendered_frame_sampler;
layout (set=0, binding = 2) uniform texture2D past_frame_texture;
layout (set=0, binding = 3) uniform sampler past_frame_sampler;
layout (set=0, binding = 4) uniform texture2D rendered_frame_depth_texture;
layout (set=0, binding = 5) uniform sampler rendered_frame_depth_sampler; 
layout (set=0, binding = 6) uniform texture2D past_frame_depth_texture;
layout (set=0, binding = 7) uniform sampler past_frame_depth_sampler; 
layout (set=0, binding = 8) uniform texture2D rendered_frame_albedo_texture;
layout (set=0, binding = 9) uniform sampler rendered_frame_albedo_sampler; 

layout (set=1, binding=0, std430) uniform Uniforms {
    mat4 invPastCameraMatrix;
    mat4 camera_matrix;
    ivec2 resolution; 
    float focal_length;
    int frame_count;
    int enable_filtering;
    float repro_percent;
    float blur_strength;
};

void main() {
    vec2 resolution = vec2(resolution.x, resolution.y);

    vec2 textureCoordinate = gl_FragCoord.xy/resolution;
    float reprojectionPercentWeighted = repro_percent;

    // Get the freshly rendered color and depth information
    vec4 renderedFrameColor = texture(sampler2D(rendered_frame_texture, rendered_frame_sampler), textureCoordinate);
    vec4 renderedFrameDepthNormals = texture(sampler2D(rendered_frame_depth_texture, rendered_frame_depth_sampler), textureCoordinate);
    float renderedFrameDepth = renderedFrameDepthNormals.a*10000;
    vec3 renderedFrameNormal = renderedFrameDepthNormals.rgb;
    vec4 renderedFrameAlbedo = texture(sampler2D(rendered_frame_albedo_texture, rendered_frame_albedo_sampler), textureCoordinate);

    // from https://www.shadertoy.com/view/ldKBzG
    vec2 offset[25];
    offset[0] = vec2(-2,-2);
    offset[1] = vec2(-1,-2);
    offset[2] = vec2(0,-2);
    offset[3] = vec2(1,-2);
    offset[4] = vec2(2,-2);
    
    offset[5] = vec2(-2,-1);
    offset[6] = vec2(-1,-1);
    offset[7] = vec2(0,-1);
    offset[8] = vec2(1,-1);
    offset[9] = vec2(2,-1);
    
    offset[10] = vec2(-2,0);
    offset[11] = vec2(-1,0);
    offset[12] = vec2(0,0);
    offset[13] = vec2(1,0);
    offset[14] = vec2(2,0);
    
    offset[15] = vec2(-2,1);
    offset[16] = vec2(-1,1);
    offset[17] = vec2(0,1);
    offset[18] = vec2(1,1);
    offset[19] = vec2(2,1);
    
    offset[20] = vec2(-2,2);
    offset[21] = vec2(-1,2);
    offset[22] = vec2(0,2);
    offset[23] = vec2(1,2);
    offset[24] = vec2(2,2);
    
    
    float kernel[25];
    kernel[0] = 1.0f/256.0f;
    kernel[1] = 1.0f/64.0f;
    kernel[2] = 3.0f/128.0f;
    kernel[3] = 1.0f/64.0f;
    kernel[4] = 1.0f/256.0f;
    
    kernel[5] = 1.0f/64.0f;
    kernel[6] = 1.0f/16.0f;
    kernel[7] = 3.0f/32.0f;
    kernel[8] = 1.0f/16.0f;
    kernel[9] = 1.0f/64.0f;
    
    kernel[10] = 3.0f/128.0f;
    kernel[11] = 3.0f/32.0f;
    kernel[12] = 9.0f/64.0f;
    kernel[13] = 3.0f/32.0f;
    kernel[14] = 3.0f/128.0f;
    
    kernel[15] = 1.0f/64.0f;
    kernel[16] = 1.0f/16.0f;
    kernel[17] = 3.0f/32.0f;
    kernel[18] = 1.0f/16.0f;
    kernel[19] = 1.0f/64.0f;
    
    kernel[20] = 1.0f/256.0f;
    kernel[21] = 1.0f/64.0f;
    kernel[22] = 3.0f/128.0f;
    kernel[23] = 1.0f/64.0f;
    kernel[24] = 1.0f/256.0f;
    
    vec4 sum = vec4(0.0);   
    float cumulative_weight = 0.0;

    for(int i=0; i<25; i++)
    { 
        vec2 offset = offset[i]*blur_strength;
        vec2 uv = (gl_FragCoord.xy+offset)/resolution;
        
        vec4 localFrameColor = texture(sampler2D(rendered_frame_texture, rendered_frame_sampler), uv);
        vec4 localFrameAlbedo = texture(sampler2D(rendered_frame_albedo_texture, rendered_frame_albedo_sampler), uv);
        vec4 delta = (renderedFrameAlbedo - localFrameAlbedo)*10000;
        float dist2 = dot(delta,delta);
        float color_weight = min(exp(-(dist2)), 1.0);
        
        vec4 localFrameNormal = texture(sampler2D(rendered_frame_depth_texture, rendered_frame_depth_sampler), uv);
        delta = (renderedFrameDepthNormals - localFrameNormal) * vec4(1.0, 1.0, 1.0, 100000.0);
        dist2 = max(dot(delta, delta), 0.0);
        float normal_weight = min(exp(-(dist2)), 1.0);
        
        float weight = color_weight*normal_weight;
        // float weight = normal_weight;
        sum += localFrameColor*weight*kernel[i];
        cumulative_weight += weight*kernel[i];
    }

    if(enable_filtering == 1) {
        renderedFrameColor = sum/cumulative_weight;
    }

    // Setup a raycast to find the worldspace position of the current pixel
    vec2 s = vec2((gl_FragCoord.x) - resolution.x/2.0f, (resolution.y - gl_FragCoord.y) - resolution.y/2.0f);
	vec3 raypos = (camera_matrix * vec4(0, 0, 0, 1)).xyz; 
	vec3 raydir = normalize(vec3(s.x/resolution.y, focal_length, s.y/resolution.y));
	raydir = (camera_matrix * vec4(raydir, 0.0)).xyz;
    vec3 worldSpacePosition = raypos + raydir * renderedFrameDepth;

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
    vec4 pastFrameDepthNormals = texture(sampler2D(past_frame_depth_texture, past_frame_depth_sampler), prevUV);
    float pastFrameDepth = pastFrameDepthNormals.a*10000;
    vec3 pastFrameNormal = pastFrameDepthNormals.rgb;
    // pastFrameColor.rgb = pow(pastFrameColor.rgb, vec3(2.2)); // Reverse the srgb color transform applied to it

    // If the camera space coordinate is outside of the previous frame then reject it.
    if (cameraSpacePosition.y < focal_length || any(greaterThan(prevUV, vec2(1))) || any(lessThan(prevUV, vec2(0)))) {
        reprojectionPercentWeighted = 0;
    }

    // Don't reproject the sky
    if(renderedFrameDepth == 0.0) {
        reprojectionPercentWeighted = 0.0;
        pastFrameDepth = 0.0;
    }

    if(pastFrameDepth == 0.0) {
        reprojectionPercentWeighted = 0.0;
    }

    if(length(pastFrameNormal - renderedFrameNormal) > 0.1) {
        reprojectionPercentWeighted *= 0.1;
    }
    
    reprojectionPercentWeighted *= min(1/abs(pastFrameDepth - renderedFrameDepth), 1.0);

    if(enable_filtering != 1) {
        reprojectionPercentWeighted = 0.0;
    }

    // Finally average out the depth and color information
    outColor = renderedFrameColor * (1.0 - reprojectionPercentWeighted) + pastFrameColor * reprojectionPercentWeighted;
    outDepth = renderedFrameDepthNormals;
    // outColor = renderedFrameColor;
    // outColor = vec4(vec3(abs(pastFrameDepth - renderedFrameDepth)), 1.0);
    // outColor = vec4(vec3(cumulative_weight), 1.0);
    // outColor = vec4(vec3(dot(vec3(camera_matrix[0][2], camera_matrix[1][2], camera_matrix[2][2]), abs(renderedFrameNormal))), 1.0);
    // outColor = renderedFrameDepthNormals;
    // outColor = vec4(renderedFrameNormal, 1.0);

    // Uncomment this code to render once a second and extrapolate between frames
    // if(frameCount % 60 == 0) { 
    //     outColor = renderedFrameColor;
    //     gl_FragDepth = renderedFrameDepth/10000;
    // } else {
    //     outColor = pastFrameColor;
    //     if(pastFrameColor.a < 0.1 || reprojectionPercentWeighted < 0.1) {
    //         outColor = vec4(1.0, 0.0, 0.0, 1.0);
    //     }
    // }

    // And apply an srgb color transform
    // outColor = pow(outColor, vec4(vec3(1.0/2.2), 1.0));

    // outColor = vec4(vec3(prevUV, 0.0), 1.0);
    // outColor = vec4(vec3(ivec3(floor(worldSpacePosition)) % ivec3(2.0)), 1.0);
    // outColor = vec4(vec3(abs(minDepthDistance)), 1.0);
    // outColor = vec4(vec3(length(abs(prevUV-textureCoordinate)) * 20), 1.0);
    // outColor = vec4(vec3(prevUV, 0.0), 1.0);
}