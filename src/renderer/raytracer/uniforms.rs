
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    resolution: [u32; 2],
    samples: u32,
    focal_length: f32,
    frame_count: u32,
    camera_matrix: todo,
    scene_size: [u32; 3],
    max_steps: u32,
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            resolution: [0; 2],
        }
    }

    pub fn update(&mut self, state: &super::State) {
        self.resolution = [state.size.width, state.size.height];
    }
}

