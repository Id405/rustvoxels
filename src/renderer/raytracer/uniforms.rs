use glam::{IVec2, IVec3, Mat4, Vec3};

use super::RenderState;
use crate::game::World;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Uniforms {
    /*
    See raytrace.frag to see the shenanagins this code is
    changing the order of these values will break things
    even if changes match in the shader
    ivec3 scene_size;
    int samples;
    ivec2 resolution;
    int frame_count;
    int max_steps;
    int octree_depth;
    float testcolor;
    float focal_length;
    */
    scene_size: IVec3,
    samples: i32,
    resolution: IVec2,
    frame_count: i32,
    max_steps: i32,
    octree_depth: i32,
    focal_length: f32,
}

unsafe impl bytemuck::Zeroable for Uniforms {}
unsafe impl bytemuck::Pod for Uniforms {}

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

        self.resolution = IVec2::new(
            render_state.size.width as i32,
            render_state.size.height as i32,
        );
        self.frame_count = render_state.frame_count as i32;
        self.focal_length = player.camera.focal_length();
        self.scene_size = IVec3::new(
            voxel_grid.width() as i32,
            voxel_grid.length() as i32,
            voxel_grid.height() as i32,
        );
        self.octree_depth = voxel_grid.get_mip_levels() as i32;
        self.max_steps = 200; // TODO; config refactor
        self.samples = 1; // TODO; config refactor
    }
}
