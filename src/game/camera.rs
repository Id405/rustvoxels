pub struct Camera {
    pub fov: f32,
}

impl Camera {
    pub fn focal_length(&self) -> f32 {
        0.5 * ((90.0 - self.fov / 2.0) * std::f32::consts::PI / 180.0).tan()
    }
}
