use std::{rc::Rc, sync::Arc};

use futures::lock::Mutex;
use glam::Vec3;
use winit::event::DeviceEvent;
use winit::event::KeyboardInput;
use winit::event::MouseButton;
use winit::event::VirtualKeyCode::*;

use self::keyboard_tracker::KeyboardTracker;

use super::{Camera, Player, Transform, World};

mod keyboard_tracker;

pub enum InputEvent {
    Keyboard(KeyboardInput),
    Mouse((f64, f64)),
    MouseButton(MouseButton),
}

pub struct GameLogic {
    world: Arc<Mutex<World>>,
    keyboard_state: KeyboardTracker,
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

        let keyboard_state = KeyboardTracker::new();

        Self {
            world,
            keyboard_state,
        }
    }

    pub async fn input_event(&mut self, event: &InputEvent) {
        let mut world = self.world.lock().await;
        let player = world
            .player
            .as_mut()
            .expect("ERROR: expected resource not found");
        match event {
            InputEvent::Keyboard(key_event) => self.keyboard_state.input_event(key_event),
            InputEvent::Mouse(delta) => {
                println!("{:?}", delta);
                player
                    .transform
                    .add_rotation(Vec3::new(delta.0 as f32, 0.0, delta.1 as f32))
            }
            InputEvent::MouseButton(_) => todo!(),
        }
    }

    pub async fn update(&mut self, delta: f32) {
        let mut world = self.world.lock().await;
        let player = world
            .player
            .as_mut()
            .expect("ERROR: expected resource not found");

        let mut move_dir = Vec3::new(0.0, 0.0, 0.0);

        if self.keyboard_state.is_pressed(W) {
            // TODO; custom keybinding so non colemak users can use this
            move_dir.y += 1.0;
        }

        if self.keyboard_state.is_pressed(R) {
            move_dir.y -= 1.0;
        }

        if self.keyboard_state.is_pressed(S) {
            move_dir.x += 1.0;
        }

        if self.keyboard_state.is_pressed(A) {
            move_dir.x -= 1.0;
        }

        if self.keyboard_state.is_pressed(Space) {
            move_dir.z += 1.0;
        }

        if self.keyboard_state.is_pressed(C) {
            move_dir.z -= 1.0;
        }

        move_dir *= delta * 10.0;

        player.transform.walk(move_dir);
    }
}
