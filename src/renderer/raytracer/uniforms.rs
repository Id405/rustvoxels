use std::sync::Arc;

use futures::lock::Mutex;
use glam::{IVec2, IVec3, Mat4, Vec3};

use super::RenderState;
use crate::game::World;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Uniforms {
    /*
    This code is a mess
    It wasted weeks of my life
    Do not edit it

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
    pub async fn new(world: Arc<Mutex<World>>, render_state: &RenderState) -> Self {
        let mut uniforms = Self::default();
        uniforms.update(world, render_state).await;
        uniforms
    }

    pub async fn update(&mut self, world: Arc<Mutex<World>>, render_state: &RenderState) {
        let world = world.lock().await;
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
        self.samples = 10; // TODO; config refactor
        if self.frame_count % 60 == 0 {
            println!("{:?}", self);
        }
    }
}
