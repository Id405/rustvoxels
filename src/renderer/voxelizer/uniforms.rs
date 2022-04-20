use crevice::std430::AsStd430;
use std::{convert::TryInto, sync::Arc, cell::RefCell, rc::Rc};

use futures::lock::Mutex;
use glam::{IVec2, IVec3, Mat4, Vec3};

use crate::{game::World, renderer::{RenderContext, texture_atlas::TextureAtlas}};

#[derive(Copy, Clone, Debug, AsStd430)]
pub struct Uniforms {
    scene_size: u32,
    model_matrix: mint::ColumnMatrix4<f32>,
}

impl Uniforms {
    pub async fn new(context: &RenderContext, world: Arc<Mutex<World>>, atlas: Rc<RefCell<TextureAtlas>>) -> Self {
        let mut uniforms = Self {
            scene_size: 0,
            model_matrix: Mat4::IDENTITY.into(),
        };
        uniforms.update(context, world, atlas).await;
        uniforms
    }

    pub async fn update(&mut self, context: &RenderContext, world: Arc<Mutex<World>>, atlas: Rc<RefCell<TextureAtlas>>) {
        let info = atlas.borrow_mut().get_info("voxelizer_attachment_world", context).unwrap();
        self.scene_size = info.size.0; // TODO change fixed scene size
    }

    pub async fn update_model_matrix(&mut self, model_matrix: Mat4) {
        self.model_matrix = model_matrix.into();
    }
}
