pub struct Camera {
    pub fov: f32,
}

impl Camera {
    pub fn focal_length(&self) -> f32 {
        1.0 / f32::tan(self.fov / 2.0 * std::f32::consts::PI / 180.0)
    }
}
