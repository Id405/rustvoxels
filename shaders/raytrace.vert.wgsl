[[block]]
struct gl_PerVertex {
    [[builtin(position)]] gl_Position: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] gl_Position: vec4<f32>;
};

var<private> perVertexStruct: gl_PerVertex = gl_PerVertex(vec4<f32>(0.0, 0.0, 0.0, 1.0), );
var<private> position1: vec3<f32>;

fn main1() {
    let _e9: vec3<f32> = position1;
    perVertexStruct.gl_Position = vec4<f32>(_e9.x, _e9.y, _e9.z, 1.0);
    return;
}

[[stage(vertex)]]
fn main([[location(0)]] position: vec3<f32>) -> VertexOutput {
    position1 = position;
    main1();
    let _e7: vec4<f32> = perVertexStruct.gl_Position;
    return VertexOutput(_e7);
}
