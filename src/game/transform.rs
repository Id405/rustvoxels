use glam::{Mat4, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};

#[derive(Debug)]
pub struct Transform {
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
}

impl Transform {
    pub fn as_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position)
            * Mat4::from_rotation_z(self.rotation.z)
            * Mat4::from_rotation_x(self.rotation.x)
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position.xzy() * Vec3::new(1.0, 1.0, -1.0))
            * Mat4::from_rotation_y(self.rotation.z)
            * Mat4::from_rotation_x(self.rotation.x)
        // Mat4::IDENTITY
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_translation(self.position.xzy() * Vec3::new(1.0, 1.0, -1.0))
            * Mat4::from_rotation_y(self.rotation.z)
            * Mat4::from_rotation_x(self.rotation.x)
            * Mat4::from_scale(self.scale)
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
        self.rotation += rotation;
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    pub fn add_scale(&mut self, scale: Vec3) {
        self.scale += scale;
    }

    pub fn walk(&mut self, movement: Vec3) {
        // Translate with respect to rotation
        let delta = (Mat4::from_rotation_z(self.rotation().z)
            * Vec4::new(movement.x, movement.y, movement.z, 1.0))
        .xyz();
        self.add_position(delta)
    }

    pub fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }
}
