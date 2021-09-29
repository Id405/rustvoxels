use glam::Mat4;

use crate::game::World;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct CameraUniform {
    transform: Mat4,
}

impl CameraUniform {
    pub fn new(world: &World) -> Self {
        let mut uniforms = Self::default();
        uniforms.update(world);
        uniforms
    }

    pub fn update(&mut self, world: &World) {
        self.transform = world
            .player
            .as_ref()
            .expect("ERROR: expected resource not found")
            .transform
            .as_matrix();
    }
}

unsafe impl bytemuck::Zeroable for CameraUniform {}
unsafe impl bytemuck::Pod for CameraUniform {}
