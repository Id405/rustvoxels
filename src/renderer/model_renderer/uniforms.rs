use crevice::std430::AsStd430;
use std::{convert::TryInto, sync::Arc};

use futures::lock::Mutex;
use glam::{IVec2, IVec3, Mat4, Vec3};

use crate::game::World;

#[derive(Copy, Clone, Debug, AsStd430)]
pub struct Uniforms {
    view_matrix: mint::ColumnMatrix4<f32>,
    model_matrix: mint::ColumnMatrix4<f32>,
}

impl Uniforms {
    pub async fn new(world: Arc<Mutex<World>>) -> Self {
        let mut uniforms = Self {
            view_matrix: Mat4::IDENTITY.into(),
            model_matrix: Mat4::IDENTITY.into(),
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
        // self.view_matrix = (Mat4::perspective_rh_gl(
        //     player.camera.fov,
        //     (player.camera.size.width / player.camera.size.height) as f32,
        //     0.1,
        //     1000.0,
        // ) * player.transform.as_matrix().inverse())
        // .into();
        self.view_matrix = (Mat4::perspective_rh_gl(
            player.camera.fov,
            (player.camera.size.width / player.camera.size.height) as f32,
            0.1,
            1000.0,
        ) * player.transform.view_matrix().inverse())
        .into();
    }

    pub async fn update_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix.into();
    }
}
