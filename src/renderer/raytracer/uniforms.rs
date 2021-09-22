use cgmath::Matrix;

use super::RenderState;
use crate::game::World;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    resolution: [i32; 2],
    samples: i32,
    focal_length: f32,
    frame_count: i32,
    world_matrix: [[f32; 4]; 4],
    scene_size: [i32; 3],
    max_steps: i32,
    octree_depth: i32,
}

impl Uniforms {
    pub fn new(world: &World, render_state: &RenderState) -> Self {
        let mut uniforms = Self::default();
        uniforms.update(world, render_state);
        uniforms
    }

    pub fn update(&mut self, world: &World, render_state: &RenderState) {
        let player = world
            .player
            .as_ref()
            .expect("ERROR: expected resource not present");
        let voxel_grid = world
            .voxel_grid
            .as_ref()
            .expect("ERROR: expected resource not present");

        self.resolution = [render_state.size.width as i32, render_state.size.height as i32];
        self.frame_count = render_state.frame_count as i32;
        self.focal_length = player.camera.focal_length();
        self.world_matrix = player.transform.as_matrix().into();
        self.scene_size = [
            voxel_grid.width() as i32,
            voxel_grid.length() as i32,
            voxel_grid.height() as i32,
        ];
        self.octree_depth = voxel_grid.get_mip_levels() as i32;
        self.max_steps = 200; // TODO; config refactor
        self.samples = 1; // TODO; config refactor
        println!("{:?}", self.scene_size);
    }
}
