use glam::IVec2;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Uniforms {
    resolution: IVec2,
}

impl Uniforms {
    pub async fn new(resolution: IVec2) -> Self {
        Self { resolution }
    }

    pub async fn update(&mut self, resolution: IVec2) {
        self.resolution = resolution;
    }
}

unsafe impl bytemuck::Zeroable for Uniforms {}
unsafe impl bytemuck::Pod for Uniforms {}
