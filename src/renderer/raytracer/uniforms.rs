use super::RenderState;
use crate::game::World;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    resolution: [u32; 2],
    samples: u32,
    focal_length: f32,
    frame_count: u32,
    world_matrix: [[f32; 4]; 4],
    scene_size: [u32; 3],
    max_steps: u32,
    octree_depth: u32,
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

        self.resolution = [render_state.size.width, render_state.size.height];
        self.frame_count = render_state.frame_count;
        self.focal_length = player.camera.focal_length();
        self.world_matrix = *player.transform.as_matrix().as_ref();
        self.scene_size = [
            voxel_grid.width() as u32,
            voxel_grid.height() as u32,
            voxel_grid.length() as u32,
        ];
        self.octree_depth = voxel_grid.get_mip_levels();
        self.max_steps = 200; // TODO; config refactor
        self.samples = 1; // TODO; config refactor
    }
}
