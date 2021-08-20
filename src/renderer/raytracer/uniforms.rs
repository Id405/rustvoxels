use crate::game;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    resolution: [u32; 2],
    samples: u32,
    focal_length: f32,
    frame_count: u32,
    camera_matrix: todo,
    scene_size: [u32; 3],
    max_steps: u32,
}

impl Uniforms {
    pub fn new(global_state: &GlobalState, world: game::World) -> Self {
        Self {
            resolution: [0; 2],
            samples: 10, //TODO for config refctor
            focal_length: 1/f32::tan(global_state.player.camera.fov/2 * f32::consts::PI/180.0),
            frame_count: (),
            camera_matrix: (),
            scene_size: (),
            max_steps: (),
        }
    }

    pub fn update(&mut self, global_state: &GlobalState) {
        self.resolution = [state.size.width, state.size.height];
    }
}

