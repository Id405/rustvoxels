use super::RenderState;
use crate::game::World;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    resolution: [u32; 2],
    samples: u32,
    focal_length: f32,
    frame_count: u32,
    world_matrix: [[f32; 4]; 4],
    scene_size: [u32; 3],
    max_steps: u32,
}

impl Uniforms {
    pub fn new(world: &World) -> Self {
        Self {
            resolution: [0; 2],
            samples: 1, //TODO; config refactor
            focal_length: world
                .player
                .as_ref()
                .expect("ERROR: expected resource not present")
                .camera
                .focal_length(),
            frame_count: 0,
            world_matrix: *world
                .player
                .as_ref()
                .expect("ERROR: expected resource not present")
                .transform
                .as_matrix()
                .as_ref(),
            scene_size: [
                world
                    .voxel_grid
                    .as_ref()
                    .as_ref()
                    .expect("ERROR: expected resource not present")
                    .width() as u32,
                world
                    .voxel_grid
                    .as_ref()
                    .as_ref()
                    .expect("ERROR: expected resource not present")
                    .height() as u32,
                world
                    .voxel_grid
                    .as_ref()
                    .as_ref()
                    .expect("ERROR: expected resource not present")
                    .length() as u32,
            ],
            max_steps: 200, // TODO; config refactor
        }
    }

    pub fn update(&mut self, world: &World, render_state: &RenderState) {
        self.resolution = [render_state.size.width, render_state.size.height];
        self.frame_count = render_state.frame_count;
        self.focal_length = world
            .player
            .as_ref()
            .expect("ERROR: expected resource not present")
            .camera
            .focal_length();
        self.world_matrix = *world
            .player
            .as_ref()
            .expect("ERROR: expected resource not present")
            .transform
            .as_matrix()
            .as_ref();
        self.scene_size = [
            world
                .voxel_grid
                .as_ref()
                .expect("ERROR: expected resource not present")
                .width() as u32,
            world
                .voxel_grid
                .as_ref()
                .expect("ERROR: expected resource not present")
                .height() as u32,
            world
                .voxel_grid
                .as_ref()
                .expect("ERROR: expected resource not present")
                .length() as u32,
        ];
        self.max_steps = 200; // TODO; config refactor
        self.samples = 1; // TODO; config refactor
    }
}
