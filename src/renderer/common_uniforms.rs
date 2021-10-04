use std::sync::Arc;

use futures::lock::Mutex;
use glam::Mat4;

use crate::game::World;

use super::RenderContext;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CameraUniform {
    transform: Mat4,
}

impl CameraUniform {
    // TODO uniform update trait
    pub async fn new(world: Arc<Mutex<World>>, context: &RenderContext) -> Self {
        let mut uniforms = Self::default();
        uniforms.update(world, context).await;
        uniforms
    }

    pub async fn update(&mut self, world: Arc<Mutex<World>>, context: &RenderContext) {
        self.transform = world
            .lock()
            .await
            .player
            .as_ref()
            .expect("ERROR: expected resource not found")
            .transform
            .as_matrix();
    }
}

unsafe impl bytemuck::Zeroable for CameraUniform {}
unsafe impl bytemuck::Pod for CameraUniform {}
