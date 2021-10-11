use winit::event::{KeyboardInput, VirtualKeyCode};

pub struct KeyboardTracker {
    state: Vec<VirtualKeyCode>,
}

impl KeyboardTracker {
    pub fn new() -> Self {
        Self { state: vec![] }
    }

    pub fn is_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.state.contains(&keycode)
    }

    pub fn input_event(&mut self, event: &KeyboardInput) {
        match event.state {
            winit::event::ElementState::Pressed => {
                if let Some(keycode) = event.virtual_keycode {
                    self.state.push(keycode);
                }
            }
            winit::event::ElementState::Released => {
                if let Some(keycode) = event.virtual_keycode {
                    self.state.retain(|key| *key != keycode);
                }
            }
        }
    }
}
