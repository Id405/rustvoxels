use crate::renderer::CameraUniform;
use glam::{Mat4, Vec3};

pub struct Transform {
    position: Vec3,
    rotation: Vec3,
}

impl Transform {
    pub fn as_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_rotation_z(self.rotation.z)
            * Mat4::from_rotation_x(self.rotation.x)
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn rotation(&self) -> Vec3 {
        self.rotation
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.rotation = rotation;
    }

    pub fn add_position(&mut self, position: Vec3) {
        self.position += position;
    }

    pub fn add_rotation(&mut self, rotation: Vec3) {
        todo!();
    }

    pub fn new(position: Vec3, rotation: Vec3) -> Self {
        Self { position, rotation }
    }
}
