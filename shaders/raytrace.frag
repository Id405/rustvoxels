#version 460
#extension GL_EXT_samplerless_texture_functions : require
#extension GL_EXT_scalar_block_layout : require

layout(location = 0) out vec4 outColor;

/*
Ok so UBO alignment in wgpu is absolutely evil.
Order matters here, 3 component vectors must be followed by a 32 bit value
*/

layout(set = 1, binding = 0, std430) uniform Raytrace {
    ivec3 scene_size; // 0 (EVEN)
	int samples; // 1 (ODD)
    ivec2 resolution; // 2 (EVEN)
    int frame_count; // 1 (ODD)
	int max_steps;  // ...
    int octree_depth;
    float focal_length;
};

layout(set = 1, binding = 1) uniform Camera {
	mat4 world_matrix;
};

layout(set = 0, binding = 0) uniform texture3D scene_texture;


#define SKYCOLOR vec3(0.9) 
#define SUNCOLOR vec3(192.0/255.0, 191.0/255.0, 173.0/255.0)
#define LIGHTCOLOR vec3(5, 0, 0)
#define LIGHTDIR vec3(0.0, 0.0, 1.0)
#define SUNSHARPNESS 2
#define SUNPOWER 4
#define SKYPOWER 0.2
#define SUNLIGHTSTRENGTH 1.0

#define PI 3.1415926535897932384626433832795

// Helper functions

uint base_hash(uvec2 p) {
    p = 1103515245U*((p >> 1U)^(p.yx));
    uint h32 = 1103515245U*((p.x)^(p.y>>3U));
    return h32^(h32 >> 16);
}

uint base_hash3(uvec3 p) {
    p = 1103515245U*((p >> 1U)^(p.zyx));
    uint h32 = 1103515245U * ((p.x)^(p.y>>3U));
    return h32^(h32 >> 16);
}

float g_seed = 0.0;

vec2 rand2(inout float seed) {
    uint n = base_hash(floatBitsToUint(vec2(seed+=1,seed+=1)));
    uvec2 rz = uvec2(n, n*48271U);
    return vec2(rz.xy & uvec2(0x7fffffffU))/float(0x7fffffff);
}

vec3 rand3(inout float seed) {
    uint n = base_hash(floatBitsToUint(vec2(seed+=1,seed+=1)));
    uvec3 rz = uvec3(n, n*16807U, n*48271U);
    return vec3(rz & uvec3(0x7fffffffU))/float(0x7fffffff);
}

vec3 rand3(vec3 seed) {
    uint n = base_hash3(floatBitsToUint(seed));
    uvec3 rz = uvec3(n, n*16807U, n*48271U);
    return vec3(rz & uvec3(0x7fffffffU))/float(0x7fffffff);
}

vec3 random_in_unit_sphere() {
    vec3 h = rand3(g_seed) * vec3(2.,6.28318530718,1.)-vec3(1.0,0.0,0.0);
    float phi = h.y;
    float r = pow(h.z, 1./3.);
	return r * vec3(sqrt(1.-h.x*h.x)*vec2(sin(phi),cos(phi)),h.x);
}

// Scatter a ray with respect to lambertian shading
vec3 scatter(vec3 n) {
	vec3 dr = random_in_unit_sphere();
	return sign(dot(dr, n))*dr;
}

// Intersect a ray with an axis aligned bounding box
bool rayAABB(vec3 rayOrigin, vec3 rayDir, vec3 boxMin, vec3 boxMax, out vec2 result, out vec3 normal) {
    vec3 rayInvDir = 1.0 / rayDir; //Can be precomputed on a set of aligned boxes
    vec3 tbot = rayInvDir * (boxMin - rayOrigin);
    vec3 ttop = rayInvDir * (boxMax - rayOrigin);
    vec3 tmin = min(ttop, tbot);
    vec3 tmax = max(ttop, tbot);
    vec2 t = max(tmin.xx, tmin.yz);
    float t0 = max(t.x, t.y);
    t = min(tmax.xx, tmax.yz);
    float t1 = min(t.x, t.y);
    result = vec2(t0, t1);
	if(t1 <= max(t0, 0.0)) return false;
	normal = -sign(rayDir)*step(tmin.yzx,tmin.xyz)*step(tmin.zxy,tmin.xyz);
    return true;
}

// Is a position inside of a bounding box?
bool insideBoundingBox(vec3 p, vec3 min, vec3 max) {
	return p.x > min.x && p.x < max.x && p.y > min.y && p.y < max.y && p.z > min.z && p.z < max.z;
}

// Is the voxel at the given position and mipmap level filled?
bool getVoxel(ivec3 c, int l) {
	return texelFetch(scene_texture, c, l).a != 0;
}

// Get the color of the voxel at a given position and mipmap level
vec3 getColor(ivec3 c, int l) {
	return texelFetch(scene_texture, clamp(c, ivec3(0), scene_size), l).rgb;
}

// The main raytracing function, the alpha channel of the vec4 that is returned is the depth
vec4 trace(vec2 p) {
	// Setup the Ray Position and Direction given the camera transformation matrix
	vec2 s = vec2(p.x - float(resolution.x)/2.0f, p.y - float(resolution.y)/2.0f);
	vec3 raypos = vec3(world_matrix[0][3], world_matrix[1][3], world_matrix[2][3]);
	// vec3 raypos = vec3(0.1, 0.0, 0.0);
	vec3 raydir = normalize(vec3(s.x/resolution.y, focal_length, s.y/resolution.y));
	raydir = (world_matrix * vec4(raydir, 0.0)).xyz;

	// Variables needed for the bounding box function
	vec3 n;
	vec2 res;

	// if(!(insideBoundingBox(raypos, vec3(0), vec3(scene_size)))) {
	// 	if(rayAABB(raypos, raydir, vec3(0, 0, 0), vec3(scene_size), res, n)) {
	// 		raypos += raydir * res.x + n * 0.00001;
	// 	} else {
	// 		// return vec4((SUNCOLOR * pow(max(dot(normalize(LIGHTDIR), raydir), 0.0), SUNSHARPNESS) * SUNPOWER + SKYCOLOR * SKYPOWER) * SUNLIGHTSTRENGTH, 10000000);
	// 		return vec4(1.0, 0.0, 0.0, 1.0);
	// 	} //TODO normal data is not needed
	// }

	int maxLevel = octree_depth-1;
	int level = maxLevel/2; // The current level in the octree

	float complexity = 0; // Used to display a complexity map, however not required for the actual rendering

	ivec3 gridPosition = ivec3(floor(raypos));

	vec3 deltaDist = abs(vec3(1)/raydir);
    ivec3 step = ivec3(sign(raydir));
	bvec3 raydirsign = greaterThan(sign(raydir), vec3(0));

	vec3 nextEdge = vec3(gridPosition & ivec3(-1 << level)) + vec3(greaterThan(raydir, vec3(0.0))) * (1 << level);
	vec3 sideDist = abs((nextEdge - raypos) * deltaDist);

	bool moved = false;

	float dist;
    vec3 normal = vec3(0.0);
	int steps = 0;

	vec3 luminance = vec3(0);
	vec3 outColor = vec3(1);
	float depth = 0;

	for(int i=0; i<200/*max_steps*/; i++) { // Begin marching the ray now
		// if(!insideBoundingBox(gridPosition, vec3(-2), scene_size + vec3(1))) { // If we aren't inside the bounding box of the scene, there is no more geometry to intersect and we can return
		// 	break;
		// }

		bool nonEmpty = getVoxel(gridPosition >> level, level); // Is the current voxel empty
		bool belowEmpty = !getVoxel(gridPosition >> (level + 1), level + 1) && level < maxLevel; // Can we move upwards an octree level?
		bool verticalMove = nonEmpty || belowEmpty; // If either we can move down or move up in the octree

		if(verticalMove) {
			complexity += int(nonEmpty); // Increment the complexity variable to keep track of a complexity map

			vec3 modifiedRayPosition = raypos;
			if(moved) {
				modifiedRayPosition = raypos + raydir * dist; // Find point of intersection between ray and the current grid position
			}

			gridPosition = ivec3(floor(modifiedRayPosition - normal * 0.0001)); // Calculate a new grid position given that information

			if(level == 0 && nonEmpty) { // If we are at the lowest level and hit a non empty grid position that means we hit scene geometry and we can scatter the ray off of it
				// return vec4(getColor(gridPosition >> level, level), 1.0); // uncomment to disable lighting

				outColor *= getColor(gridPosition >> level, level);
				// outColor *= 0.5; // Disable color and only view lighting

				if(depth == 0) { // Update the depth variable to store the distance to the first intersection with the scene geometry
					depth = dist;
					// outNormal = vec4(normal, 1.0);
				}

				modifiedRayPosition += normal * 0.01; // Step off of the scene geometry slightly to avoid getting stuck inside of it
				
				raypos = modifiedRayPosition; // Update the ray position, ray direction and the values that depend on it
				raydir = scatter(normal);
				deltaDist = abs(vec3(1)/raydir);
				step = ivec3(sign(raydir));
				raydirsign = greaterThan(sign(raydir), vec3(0));
				dist = 0; // Reset the distance to zero
			}

			level -= int(nonEmpty); // If we can move down, move down
			level = max(0, level);
			level += int(!nonEmpty); // If we can move up, move up
			
			// Recalculate the variables dependent on grid position
			nextEdge = vec3(gridPosition & ivec3(-1 << level)) + vec3(greaterThan(raydir, vec3(0.0))) * (1 << level);
			sideDist = abs((nextEdge - modifiedRayPosition) * deltaDist);

			if(moved) { // Accumulate the distance values
				sideDist += dist;
			}
		}

		if(!verticalMove) { // If we aren't moving vertically, move horizontally
			float minTime = min(sideDist.x, min(sideDist.y, sideDist.z));
			dist = minTime;

			bvec3 mask = lessThanEqual(sideDist.xyz, min(sideDist.yzx, sideDist.zxy));
			ivec3 vstep = ivec3(mix(-1, 1 << level, raydirsign.x), mix(-1, 1 << level, raydirsign.y), mix(-1, 1 << level, raydirsign.z));
			gridPosition = (gridPosition & ivec3(-1 << level)) + ivec3(mask) * vstep;
			sideDist += vec3(mask) * deltaDist * vec3(1 << level);
			normal = vec3(mask) * -step;
			moved = true;
		}

		steps = i;
	}

	// return vec4(vec3(float(steps)/max_steps), 1.0); // Return how many steps it took to render this pixel
	// return vec4(outColor, 1); // Return scene lit only using anti-aliasing
	// return vec4(vec3(complexity/(maxLevel * 4)), 1); // Return complexity map
	// return vec4(vec3(dist/128), 1); // Return distance map
	return vec4(outColor * (SUNCOLOR * pow(max(dot(normalize(LIGHTDIR), raydir), 0.0), SUNSHARPNESS) * SUNPOWER + SKYCOLOR * SKYPOWER + luminance), 1.0); // Return fully lit scene
	// return vec4(vec3(raydir.x, raydir.y, 0), 1.0);
}

void mainImage(in vec2 fragCoord )
{
	// Initialize global seed for RNG
	g_seed = float(base_hash(floatBitsToUint(fragCoord + float(frame_count)/240)))/float(0xffffffffU);

	vec4 color = vec4(0.0);

	// Render the scenes samples
	for(int i=0; i < samples; i++) {
		vec2 p = fragCoord;
		p.y = resolution.y - p.y; // Flip image vertically because ofFbo flips images vertically for some reason
		vec4 col = trace(p);

		color += vec4(col.rgb, 1.0); // Accumulate color average
	}

	color /= float(samples); // Average color
	outColor = color;
	// gl_FragDepth /= float(samples); // Average depth
	//outNormal = vec4(1.0, 0.0, 0.0, 1.0);
}

void main() {
	mainImage(gl_FragCoord.xy);
	// outColor = vec4(vec3(testcolor), 1.0);
}