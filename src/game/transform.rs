use cgmath::prelude::*;

pub struct Transform {
    position: Vector3<f32>,
    rotation: Rotation3<f32>,
}

impl transform {
    pub fn as_matrix(&self) -> Matrix4 {
        todo!()
    }

    pub fn position(&self) -> Vector3 {
        self.position;
    }

    pub fn rotation(&self) -> Rotation3<f32> {
        self.rotation;
    }

    pub fn set_position(&mut self, position: Vector3) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, rotation: Rotation3) {
        self.rotation = rotation;
    }

    pub fn add_position(&mut self, position: Vector3) {
        self.position += position;
    }

    pub fn add_rotation(&mut self, rotation: Rotation3) {
        self.rotation += rotation;
    }
}