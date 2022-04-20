use crevice::std430::AsStd430;
use std::{convert::TryInto, sync::Arc};

use futures::lock::Mutex;
use glam::{IVec2, IVec3, Mat4, Vec3};

use crate::game::World;

#[derive(Copy, Clone, Debug, AsStd430)]
pub struct Uniforms {
    scene_size: u32,
    model_matrix: mint::ColumnMatrix4<f32>,
}

impl Uniforms {
    pub async fn new(world: Arc<Mutex<World>>) -> Self {
        let mut uniforms = Self {
            scene_size: 0,
            model_matrix: Mat4::IDENTITY.into(),
        };
        uniforms.update(world).await;
        uniforms
    }

    pub async fn update(&mut self, world: Arc<Mutex<World>>) {
        self.scene_size = 512; // TODO change fixed scene size
    }

    pub async fn update_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix.into();
    }
}
