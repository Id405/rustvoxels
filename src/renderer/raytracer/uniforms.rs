use crevice::std430::AsStd430;
use std::{convert::TryInto, sync::Arc};

use futures::lock::Mutex;
use glam::{IVec2, IVec3, Mat4, Vec3};

use crate::game::World;

#[derive(Copy, Clone, Debug, AsStd430)]
pub struct Uniforms {
    camera_matrix: mint::ColumnMatrix4<f32>,
    scene_size: mint::Vector3<i32>,
    resolution: mint::Vector2<i32>,
    samples: i32,
    primary_ray_only: i32,
    frame_count: i32,
    max_steps: i32,
    octree_depth: i32,
    focal_length: f32,
}

impl Uniforms {
    pub async fn new(world: Arc<Mutex<World>>) -> Self {
        let mut uniforms = Self {
            scene_size: mint::Vector3 { x: 0, y: 0, z: 0 },
            resolution: mint::Vector2 { x: 0, y: 0 },
            samples: 0,
            frame_count: 0,
            max_steps: 0,
            octree_depth: 0,
            focal_length: 0.0,
            primary_ray_only: 0,
            camera_matrix: Mat4::IDENTITY.into(),
        };
        uniforms.update(world).await;
        uniforms
    }

    pub async fn update(&mut self, world: Arc<Mutex<World>>) {
        let world = world.lock().await;
        let player = world
            .player
            .as_ref()
            .expect("ERROR: expected resource not present");
        let voxel_grid = world
            .voxel_grid
            .as_ref()
            .expect("ERROR: expected resource not present");
        let config = world
            .config
            .as_ref()
            .expect("ERROR: expected resource not present");

        self.resolution = IVec2::new(
            player.camera.size.width as i32,
            player.camera.size.height as i32,
        )
        .into();
        self.frame_count = player.camera.frame_count as i32;
        self.focal_length = player.camera.focal_length();
        self.scene_size = IVec3::new(
            voxel_grid.width() as i32,
            voxel_grid.length() as i32,
            voxel_grid.height() as i32,
        )
        .into();
        self.octree_depth = voxel_grid.get_mip_levels() as i32;
        self.max_steps = config.get_var("renderer_raytracer_max_steps").unwrap().as_i32(); // TODO; config refactor
        self.samples = config.get_var("renderer_raytracer_samples").unwrap().as_i32(); // TODO; config refactor
        self.camera_matrix = player.transform.as_matrix().into();
        self.primary_ray_only = config.get_var("renderer_raytracer_do_lighting").unwrap().as_i32();
    }
}
