use crevice::std430::AsStd430;
use std::sync::Arc;

use futures::lock::Mutex;
use glam::{IVec2, Mat4};

use crate::game::World;

#[repr(C)]
#[derive(Copy, Clone, Debug, AsStd430)]
pub struct Uniforms {
    inverse_past_camera_matrix: mint::ColumnMatrix4<f32>,
    camera_matrix: mint::ColumnMatrix4<f32>,
    resolution: mint::Vector2<i32>,
    focal_length: f32,
    frame_count: i32,
}

impl Uniforms {
    pub async fn new(world: Arc<Mutex<World>>) -> Self {
        let mut new = Self {
            inverse_past_camera_matrix: Mat4::default().into(),
            camera_matrix: Mat4::default().into(),
            resolution: mint::Vector2 { x: 1, y: 1 },
            focal_length: 0.5,
            frame_count: 0,
        };
        new.update(world).await;
        new
    }

    pub async fn update(&mut self, world: Arc<Mutex<World>>) {
        let world_lock = world.lock().await;
        let player = world_lock
            .player
            .as_ref()
            .expect("ERROR: expected resource not found");

        self.inverse_past_camera_matrix = Mat4::from(self.camera_matrix).inverse().into();
        self.camera_matrix = player.transform.as_matrix().into();
        self.resolution = IVec2::new(
            player.camera.size.width as i32,
            player.camera.size.height as i32,
        )
        .into();
        self.focal_length = player.camera.focal_length();
        self.frame_count = player.camera.frame_count as i32;
    }
}
