[[block]]
struct Raytrace {
    resolution: vec2<i32>;
    samples: i32;
    focal_length: f32;
    frame_count: i32;
    world_matrix: mat4x4<f32>;
    scene_size: vec3<i32>;
    max_steps: i32;
    octree_depth: i32;
};

var<private> g_seed: f32;
[[group(0), binding(0)]]
var scene_texture: texture_3d<f32>;
[[group(1), binding(0)]]
var<uniform> _: Raytrace;
var<private> outColor1: vec4<f32>;
var<private> gl_FragCoord1: vec4<f32>;

fn base_hashvu2_(p: ptr<function, vec2<u32>>) -> u32 {
    var h32_: u32;

    let _e63: vec2<u32> = *p;
    let _e67: vec2<u32> = *p;
    p = (vec2<u32>(1103515245u) * ((_e63 >> vec2<u32>(vec2<u32>(1u))) ^ _e67.yx));
    let _e73: u32 = p[0u];
    let _e75: u32 = p[1u];
    h32_ = (1103515245u * (_e73 ^ (_e75 >> bitcast<u32>(3u))));
    let _e80: u32 = h32_;
    let _e81: u32 = h32_;
    return (_e80 ^ (_e81 >> bitcast<u32>(16)));
}

fn rand3f1_(seed: ptr<function, f32>) -> vec3<f32> {
    var n: u32;
    var param: vec2<u32>;
    var rz: vec3<u32>;

    let _e65: f32 = *seed;
    let _e66: f32 = (_e65 + 1.0);
    seed = _e66;
    let _e67: f32 = *seed;
    let _e68: f32 = (_e67 + 1.0);
    seed = _e68;
    param = vec2<u32>(vec2<f32>(_e66, _e68));
    let _e71: u32 = base_hashvu2_(param);
    n = _e71;
    let _e72: u32 = n;
    let _e73: u32 = n;
    let _e75: u32 = n;
    rz = vec3<u32>(_e72, (_e73 * 16807u), (_e75 * 48271u));
    let _e78: vec3<u32> = rz;
    return (vec3<f32>((_e78 & vec3<u32>(2147483647u, 2147483647u, 2147483647u))) / vec3<f32>(2147483648.0));
}

fn random_in_unit_sphere() -> vec3<f32> {
    var h: vec3<f32>;
    var param1: f32;
    var phi: f32;
    var r: f32;

    let _e65: f32 = g_seed;
    param1 = _e65;
    let _e66: vec3<f32> = rand3f1_(param1);
    let _e67: f32 = param1;
    g_seed = _e67;
    h = ((_e66 * vec3<f32>(2.0, 6.2831854820251465, 1.0)) - vec3<f32>(1.0, 0.0, 0.0));
    let _e71: f32 = h[1u];
    phi = _e71;
    let _e73: f32 = h[2u];
    r = pow(_e73, 0.3333333432674408);
    let _e75: f32 = r;
    let _e77: f32 = h[0u];
    let _e79: f32 = h[0u];
    let _e83: f32 = phi;
    let _e85: f32 = phi;
    let _e88: vec2<f32> = (vec2<f32>(sin(_e83), cos(_e85)) * sqrt((1.0 - (_e77 * _e79))));
    let _e90: f32 = h[0u];
    return (vec3<f32>(_e88.x, _e88.y, _e90) * _e75);
}

fn scattervf3_(n1: ptr<function, vec3<f32>>) -> vec3<f32> {
    var dr: vec3<f32>;

    let _e63: vec3<f32> = random_in_unit_sphere();
    dr = _e63;
    let _e64: vec3<f32> = dr;
    let _e65: vec3<f32> = *n1;
    let _e68: vec3<f32> = dr;
    return (_e68 * sign(dot(_e64, _e65)));
}

fn getColorvi3i1_(c: ptr<function, vec3<i32>>, l: ptr<function, i32>) -> vec3<f32> {
    let _e63: vec3<i32> = *c;
    let _e65: vec3<i32> = _.scene_size;
    let _e67: i32 = *l;
    let _e68: vec4<f32> = textureLoad(scene_texture, clamp(_e63, vec3<i32>(0, 0, 0), _e65), _e67);
    return _e68.xyz;
}

fn getVoxelvi3i1_(c1: ptr<function, vec3<i32>>, l1: ptr<function, i32>) -> bool {
    let _e63: vec3<i32> = *c1;
    let _e64: i32 = *l1;
    let _e65: vec4<f32> = textureLoad(scene_texture, _e63, _e64);
    return (_e65.w != 0.0);
}

fn rayAABBvf3vf3vf3vf3vf2vf3_(rayOrigin: ptr<function, vec3<f32>>, rayDir: ptr<function, vec3<f32>>, boxMin: ptr<function, vec3<f32>>, boxMax: ptr<function, vec3<f32>>, result: ptr<function, vec2<f32>>, normal: ptr<function, vec3<f32>>) -> bool {
    var rayInvDir: vec3<f32>;
    var tbot: vec3<f32>;
    var ttop: vec3<f32>;
    var tmin: vec3<f32>;
    var tmax: vec3<f32>;
    var t: vec2<f32>;
    var t0_: f32;
    var t1_: f32;

    let _e75: vec3<f32> = *rayDir;
    rayInvDir = (vec3<f32>(1.0) / _e75);
    let _e78: vec3<f32> = rayInvDir;
    let _e79: vec3<f32> = *boxMin;
    let _e80: vec3<f32> = *rayOrigin;
    tbot = (_e78 * (_e79 - _e80));
    let _e83: vec3<f32> = rayInvDir;
    let _e84: vec3<f32> = *boxMax;
    let _e85: vec3<f32> = *rayOrigin;
    ttop = (_e83 * (_e84 - _e85));
    let _e88: vec3<f32> = ttop;
    let _e89: vec3<f32> = tbot;
    tmin = min(_e88, _e89);
    let _e91: vec3<f32> = ttop;
    let _e92: vec3<f32> = tbot;
    tmax = max(_e91, _e92);
    let _e94: vec3<f32> = tmin;
    let _e96: vec3<f32> = tmin;
    t = max(_e94.xx, _e96.yz);
    let _e100: f32 = t[0u];
    let _e102: f32 = t[1u];
    t0_ = max(_e100, _e102);
    let _e104: vec3<f32> = tmax;
    let _e106: vec3<f32> = tmax;
    t = min(_e104.xx, _e106.yz);
    let _e110: f32 = t[0u];
    let _e112: f32 = t[1u];
    t1_ = min(_e110, _e112);
    let _e114: f32 = t0_;
    let _e115: f32 = t1_;
    result = vec2<f32>(_e114, _e115);
    let _e117: f32 = t1_;
    let _e118: f32 = t0_;
    if ((_e117 <= max(_e118, 0.0))) {
        return false;
    }
    let _e121: vec3<f32> = *rayDir;
    let _e124: vec3<f32> = tmin;
    let _e126: vec3<f32> = tmin;
    let _e129: vec3<f32> = tmin;
    let _e131: vec3<f32> = tmin;
    normal = ((-(sign(_e121)) * step(_e124.yzx, _e126)) * step(_e129.zxy, _e131));
    return true;
}

fn insideBoundingBoxvf3vf3vf3_(p1: ptr<function, vec3<f32>>, min: ptr<function, vec3<f32>>, max: ptr<function, vec3<f32>>) -> bool {
    var phi_269_: bool;
    var phi_277_: bool;
    var phi_285_: bool;
    var phi_293_: bool;
    var phi_301_: bool;

    let _e65: f32 = p1[0u];
    let _e67: f32 = min[0u];
    let _e68: bool = (_e65 > _e67);
    phi_269_ = _e68;
    if (_e68) {
        let _e70: f32 = p1[0u];
        let _e72: f32 = max[0u];
        phi_269_ = (_e70 < _e72);
    }
    let _e75: bool = phi_269_;
    phi_277_ = _e75;
    if (_e75) {
        let _e77: f32 = p1[1u];
        let _e79: f32 = min[1u];
        phi_277_ = (_e77 > _e79);
    }
    let _e82: bool = phi_277_;
    phi_285_ = _e82;
    if (_e82) {
        let _e84: f32 = p1[1u];
        let _e86: f32 = max[1u];
        phi_285_ = (_e84 < _e86);
    }
    let _e89: bool = phi_285_;
    phi_293_ = _e89;
    if (_e89) {
        let _e91: f32 = p1[2u];
        let _e93: f32 = min[2u];
        phi_293_ = (_e91 > _e93);
    }
    let _e96: bool = phi_293_;
    phi_301_ = _e96;
    if (_e96) {
        let _e98: f32 = p1[2u];
        let _e100: f32 = max[2u];
        phi_301_ = (_e98 < _e100);
    }
    let _e103: bool = phi_301_;
    return _e103;
}

fn tracevf2_(p2: ptr<function, vec2<f32>>) -> vec4<f32> {
    var s: vec2<f32>;
    var raypos: vec3<f32>;
    var raydir: vec3<f32>;
    var param2: vec3<f32>;
    var param3: vec3<f32>;
    var param4: vec3<f32>;
    var res: vec2<f32>;
    var n2: vec3<f32>;
    var param5: vec3<f32>;
    var param6: vec3<f32>;
    var param7: vec3<f32>;
    var param8: vec3<f32>;
    var param9: vec2<f32>;
    var param10: vec3<f32>;
    var maxLevel: i32;
    var level: i32;
    var complexity: f32;
    var gridPosition: vec3<i32>;
    var deltaDist: vec3<f32>;
    var step: vec3<i32>;
    var raydirsign: vec3<bool>;
    var nextEdge: vec3<f32>;
    var sideDist: vec3<f32>;
    var moved: bool;
    var normal1: vec3<f32>;
    var steps: i32;
    var luminance: vec3<f32>;
    var outColor: vec3<f32>;
    var depth: f32;
    var i: i32;
    var param11: vec3<f32>;
    var param12: vec3<f32>;
    var param13: vec3<f32>;
    var nonEmpty: bool;
    var param14: vec3<i32>;
    var param15: i32;
    var belowEmpty: bool;
    var param16: vec3<i32>;
    var param17: i32;
    var verticalMove: bool;
    var modifiedRayPosition: vec3<f32>;
    var dist: f32;
    var param18: vec3<i32>;
    var param19: i32;
    var param20: vec3<f32>;
    var minTime: f32;
    var mask: vec3<bool>;
    var vstep: vec3<i32>;

    let _e111: f32 = p2[0u];
    let _e114: i32 = _.resolution[0u];
    let _e119: f32 = p2[1u];
    let _e122: i32 = _.resolution[1u];
    s = vec2<f32>((_e111 - (f32(_e114) / 2.0)), (_e119 - (f32(_e122) / 2.0)));
    let _e128: mat4x4<f32> = _.world_matrix;
    raypos = (_e128 * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    let _e132: f32 = s[0u];
    let _e135: i32 = _.resolution[1u];
    let _e139: f32 = _.focal_length;
    let _e141: f32 = s[1u];
    let _e144: i32 = _.resolution[1u];
    raydir = normalize(vec3<f32>((_e132 / f32(_e135)), _e139, (_e141 / f32(_e144))));
    let _e150: mat4x4<f32> = _.world_matrix;
    let _e151: vec3<f32> = raydir;
    raydir = (_e150 * vec4<f32>(_e151.x, _e151.y, _e151.z, 0.0)).xyz;
    let _e159: vec3<i32> = _.scene_size;
    let _e161: vec3<f32> = raypos;
    param2 = _e161;
    param3 = vec3<f32>(0.0, 0.0, 0.0);
    param4 = vec3<f32>(_e159);
    let _e162: bool = insideBoundingBoxvf3vf3vf3_(param2, param3, param4);
    if (!(_e162)) {
        let _e165: vec3<i32> = _.scene_size;
        let _e167: vec3<f32> = raypos;
        param5 = _e167;
        let _e168: vec3<f32> = raydir;
        param6 = _e168;
        param7 = vec3<f32>(0.0, 0.0, 0.0);
        param8 = vec3<f32>(_e165);
        let _e169: bool = rayAABBvf3vf3vf3vf3vf2vf3_(param5, param6, param7, param8, param9, param10);
        let _e170: vec2<f32> = param9;
        res = _e170;
        let _e171: vec3<f32> = param10;
        n2 = _e171;
        if (_e169) {
            let _e172: vec3<f32> = raydir;
            let _e174: f32 = res[0u];
            let _e176: vec3<f32> = n2;
            let _e179: vec3<f32> = raypos;
            raypos = (_e179 + ((_e172 * _e174) + (_e176 * 0.000009999999747378752)));
        } else {
            let _e181: vec3<f32> = raydir;
            let _e188: vec3<f32> = ((((vec3<f32>(0.7529411911964417, 0.7490196228027344, 0.6784313917160034) * pow(max(dot(vec3<f32>(0.0, 0.6000000238418579, 0.800000011920929), _e181), 0.0), 2.0)) * 4.0) + vec3<f32>(0.18000000715255737, 0.18000000715255737, 0.18000000715255737)) * 1.0);
            return vec4<f32>(_e188.x, _e188.y, _e188.z, 10000000.0);
        }
    }
    let _e194: i32 = _.octree_depth;
    maxLevel = (_e194 - 1);
    let _e196: i32 = maxLevel;
    level = (_e196 / 2);
    complexity = 0.0;
    let _e198: vec3<f32> = raypos;
    gridPosition = vec3<i32>(floor(_e198));
    let _e201: vec3<f32> = raydir;
    deltaDist = abs((vec3<f32>(1.0, 1.0, 1.0) / _e201));
    let _e204: vec3<f32> = raydir;
    step = vec3<i32>(sign(_e204));
    let _e207: vec3<f32> = raydir;
    raydirsign = (sign(_e207) > vec3<f32>(0.0, 0.0, 0.0));
    let _e210: vec3<i32> = gridPosition;
    let _e211: i32 = level;
    let _e217: vec3<f32> = raydir;
    let _e220: i32 = level;
    nextEdge = (vec3<f32>((_e210 & vec3<i32>((-1 << bitcast<u32>(_e211))))) + (select(vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), (_e217 > vec3<f32>(0.0, 0.0, 0.0))) * f32((1 << bitcast<u32>(_e220)))));
    let _e226: vec3<f32> = nextEdge;
    let _e227: vec3<f32> = raypos;
    let _e229: vec3<f32> = deltaDist;
    sideDist = abs(((_e226 - _e227) * _e229));
    moved = false;
    normal1 = vec3<f32>(0.0, 0.0, 0.0);
    steps = 0;
    luminance = vec3<f32>(0.0, 0.0, 0.0);
    outColor = vec3<f32>(1.0, 1.0, 1.0);
    depth = 0.0;
    i = 0;
    loop {
        let _e232: i32 = i;
        let _e234: i32 = _.max_steps;
        if ((_e232 < _e234)) {
        } else {
            break;
        }
        let _e236: vec3<i32> = gridPosition;
        let _e239: vec3<i32> = _.scene_size;
        param11 = vec3<f32>(_e236);
        param12 = vec3<f32>(-2.0, -2.0, -2.0);
        param13 = (vec3<f32>(_e239) + vec3<f32>(1.0, 1.0, 1.0));
        let _e242: bool = insideBoundingBoxvf3vf3vf3_(param11, param12, param13);
        if (!(_e242)) {
            break;
        }
        let _e244: vec3<i32> = gridPosition;
        let _e245: i32 = level;
        param14 = (_e244 >> vec3<u32>(vec3<i32>(_e245)));
        let _e249: i32 = level;
        param15 = _e249;
        let _e250: bool = getVoxelvi3i1_(param14, param15);
        nonEmpty = _e250;
        let _e251: vec3<i32> = gridPosition;
        let _e252: i32 = level;
        let _e257: i32 = level;
        param16 = (_e251 >> vec3<u32>(vec3<i32>((_e252 + 1))));
        param17 = (_e257 + 1);
        let _e259: bool = getVoxelvi3i1_(param16, param17);
        let _e261: i32 = level;
        let _e262: i32 = maxLevel;
        belowEmpty = (!(_e259) && (_e261 < _e262));
        let _e265: bool = nonEmpty;
        let _e266: bool = belowEmpty;
        verticalMove = (_e265 || _e266);
        let _e268: bool = verticalMove;
        if (_e268) {
            let _e269: bool = nonEmpty;
            let _e272: f32 = complexity;
            complexity = (_e272 + f32(select(0, 1, _e269)));
            let _e274: vec3<f32> = raypos;
            modifiedRayPosition = _e274;
            let _e275: bool = moved;
            if (_e275) {
                let _e276: vec3<f32> = raypos;
                let _e277: vec3<f32> = raydir;
                let _e278: f32 = dist;
                modifiedRayPosition = (_e276 + (_e277 * _e278));
            }
            let _e281: vec3<f32> = modifiedRayPosition;
            let _e282: vec3<f32> = normal1;
            gridPosition = vec3<i32>(floor((_e281 - (_e282 * 0.00009999999747378752))));
            let _e287: i32 = level;
            let _e289: bool = nonEmpty;
            if (((_e287 == 0) && _e289)) {
                let _e291: vec3<i32> = gridPosition;
                let _e292: i32 = level;
                param18 = (_e291 >> vec3<u32>(vec3<i32>(_e292)));
                let _e296: i32 = level;
                param19 = _e296;
                let _e297: vec3<f32> = getColorvi3i1_(param18, param19);
                let _e298: vec3<f32> = outColor;
                outColor = (_e298 * _e297);
                let _e300: f32 = depth;
                if ((_e300 == 0.0)) {
                    let _e302: f32 = dist;
                    depth = _e302;
                }
                let _e303: vec3<f32> = normal1;
                let _e305: vec3<f32> = modifiedRayPosition;
                modifiedRayPosition = (_e305 + (_e303 * 0.009999999776482582));
                let _e307: vec3<f32> = modifiedRayPosition;
                raypos = _e307;
                let _e308: vec3<f32> = normal1;
                param20 = _e308;
                let _e309: vec3<f32> = scattervf3_(param20);
                raydir = _e309;
                let _e310: vec3<f32> = raydir;
                deltaDist = abs((vec3<f32>(1.0, 1.0, 1.0) / _e310));
                let _e313: vec3<f32> = raydir;
                step = vec3<i32>(sign(_e313));
                let _e316: vec3<f32> = raydir;
                raydirsign = (sign(_e316) > vec3<f32>(0.0, 0.0, 0.0));
                dist = 0.0;
            }
            let _e319: bool = nonEmpty;
            let _e321: i32 = level;
            level = (_e321 - select(0, 1, _e319));
            let _e323: i32 = level;
            level = max(0, _e323);
            let _e325: bool = nonEmpty;
            let _e328: i32 = level;
            level = (_e328 + select(0, 1, !(_e325)));
            let _e330: vec3<i32> = gridPosition;
            let _e331: i32 = level;
            let _e337: vec3<f32> = raydir;
            let _e340: i32 = level;
            nextEdge = (vec3<f32>((_e330 & vec3<i32>((-1 << bitcast<u32>(_e331))))) + (select(vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), (_e337 > vec3<f32>(0.0, 0.0, 0.0))) * f32((1 << bitcast<u32>(_e340)))));
            let _e346: vec3<f32> = nextEdge;
            let _e347: vec3<f32> = modifiedRayPosition;
            let _e349: vec3<f32> = deltaDist;
            sideDist = abs(((_e346 - _e347) * _e349));
            let _e352: bool = moved;
            if (_e352) {
                let _e353: f32 = dist;
                let _e354: vec3<f32> = sideDist;
                sideDist = (_e354 + vec3<f32>(_e353));
            }
        }
        let _e357: bool = verticalMove;
        if (!(_e357)) {
            let _e360: f32 = sideDist[0u];
            let _e362: f32 = sideDist[1u];
            let _e364: f32 = sideDist[2u];
            minTime = min(_e360, min(_e362, _e364));
            let _e367: f32 = minTime;
            dist = _e367;
            let _e368: vec3<f32> = sideDist;
            let _e369: vec3<f32> = sideDist;
            let _e371: vec3<f32> = sideDist;
            mask = (_e368 <= min(_e369.yzx, _e371.zxy));
            let _e375: i32 = level;
            let _e379: bool = raydirsign[0u];
            let _e381: i32 = level;
            let _e385: bool = raydirsign[1u];
            let _e387: i32 = level;
            let _e391: bool = raydirsign[2u];
            vstep = vec3<i32>(select(-1, (1 << bitcast<u32>(_e375)), _e379), select(-1, (1 << bitcast<u32>(_e381)), _e385), select(-1, (1 << bitcast<u32>(_e387)), _e391));
            let _e394: vec3<i32> = gridPosition;
            let _e395: i32 = level;
            let _e400: vec3<bool> = mask;
            let _e402: vec3<i32> = vstep;
            gridPosition = ((_e394 & vec3<i32>((-1 << bitcast<u32>(_e395)))) + (select(vec3<i32>(0, 0, 0), vec3<i32>(1, 1, 1), _e400) * _e402));
            let _e405: vec3<bool> = mask;
            let _e407: vec3<f32> = deltaDist;
            let _e409: i32 = level;
            let _e415: vec3<f32> = sideDist;
            sideDist = (_e415 + ((select(vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), _e405) * _e407) * vec3<f32>(f32((1 << bitcast<u32>(_e409))))));
            let _e417: vec3<bool> = mask;
            let _e419: vec3<i32> = step;
            normal1 = (select(vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 1.0), _e417) * vec3<f32>(-(_e419)));
            moved = true;
        }
        let _e423: i32 = i;
        steps = _e423;
        continuing {
            let _e424: i32 = i;
            i = (_e424 + 1);
        }
    }
    let _e426: f32 = depth;
    if ((_e426 == 0.0)) {
        depth = 10000000.0;
    }
    let _e428: vec3<f32> = outColor;
    let _e429: vec3<f32> = raydir;
    let _e436: vec3<f32> = luminance;
    let _e438: vec3<f32> = (_e428 * ((((vec3<f32>(0.7529411911964417, 0.7490196228027344, 0.6784313917160034) * pow(max(dot(vec3<f32>(0.0, 0.6000000238418579, 0.800000011920929), _e429), 0.0), 2.0)) * 4.0) + vec3<f32>(0.18000000715255737, 0.18000000715255737, 0.18000000715255737)) + _e436));
    let _e439: f32 = depth;
    let _e441: f32 = res[0u];
    return vec4<f32>(_e438.x, _e438.y, _e438.z, (_e439 + _e441));
}

fn mainImagevf2_(fragCoord: ptr<function, vec2<f32>>) {
    var param21: vec2<u32>;
    var i1: i32;
    var p3: vec2<f32>;
    var col: vec4<f32>;
    var param22: vec2<f32>;

    let _e67: vec2<f32> = *fragCoord;
    let _e69: i32 = _.frame_count;
    param21 = vec2<u32>((_e67 + vec2<f32>((f32(_e69) / 240.0))));
    let _e75: u32 = base_hashvu2_(param21);
    g_seed = (f32(_e75) / 4294967296.0);
    i1 = 0;
    loop {
        let _e78: i32 = i1;
        let _e80: i32 = _.samples;
        if ((_e78 < _e80)) {
        } else {
            break;
        }
        let _e82: vec2<f32> = *fragCoord;
        p3 = _e82;
        let _e85: i32 = _.resolution[1u];
        let _e88: f32 = p3[1u];
        p3[1u] = (f32(_e85) - _e88);
        let _e91: vec2<f32> = p3;
        param22 = _e91;
        let _e92: vec4<f32> = tracevf2_(param22);
        col = _e92;
        let _e93: vec4<f32> = col;
        let _e94: vec3<f32> = _e93.xyz;
        let _e99: vec4<f32> = outColor1;
        outColor1 = (_e99 + vec4<f32>(_e94.x, _e94.y, _e94.z, 1.0));
        continuing {
            let _e101: i32 = i1;
            i1 = (_e101 + 1);
        }
    }
    let _e104: i32 = _.samples;
    let _e106: vec4<f32> = outColor1;
    outColor1 = (_e106 / vec4<f32>(f32(_e104)));
    return;
}

fn main1() {
    var param23: vec2<f32>;

    g_seed = 0.0;
    let _e62: vec4<f32> = gl_FragCoord1;
    param23 = _e62.xy;
    mainImagevf2_(param23);
    return;
}

[[stage(fragment)]]
fn main([[builtin(position)]] gl_FragCoord: vec4<f32>) -> [[location(0)]] vec4<f32> {
    gl_FragCoord1 = gl_FragCoord;
    main1();
    let _e3: vec4<f32> = outColor1;
    return _e3;
}
