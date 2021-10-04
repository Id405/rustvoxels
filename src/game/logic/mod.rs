use std::{rc::Rc, sync::Arc};

use futures::lock::Mutex;
use glam::Vec3;
use winit::event::KeyboardInput;

use super::{Camera, Player, Transform, World};

pub struct GameLogic {
    world: Arc<Mutex<World>>,
}

impl GameLogic {
    pub async fn new(world: Arc<Mutex<World>>) -> Self {
        {
            let mut world_lock = world.lock().await;

            world_lock.player = Some(Player {
                transform: Transform::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.5, 0.0, 0.0)),
                camera: Camera { fov: 90.0 },
            });
        }

        Self { world }
    }

    pub fn input_event(&mut self, event: &KeyboardInput) {
        match event.virtual_keycode {
            W => 
            _ => (),
        };
    }
}
