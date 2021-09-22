use cgmath::{Basis3, Matrix4, SquareMatrix, Vector3};

pub struct Transform {
    position: Vector3<f32>,
    rotation: Basis3<f32>,
}

impl Transform {
    pub fn as_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.position)
    }

    pub fn position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn rotation(&self) -> Basis3<f32> {
        self.rotation
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, rotation: Basis3<f32>) {
        self.rotation = rotation;
    }

    pub fn add_position(&mut self, position: Vector3<f32>) {
        self.position += position;
    }

    pub fn add_rotation(&mut self, rotation: Basis3<f32>) {
        todo!();
    }

    pub fn new(position: Vector3<f32>, rotation: Basis3<f32>) -> Self {
        Self { position, rotation }
    }
}
