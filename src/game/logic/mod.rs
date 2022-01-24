use std::{rc::Rc, sync::Arc};

use futures::lock::Mutex;
use glam::Vec3;
use winit::dpi::PhysicalSize;
use winit::event::DeviceEvent;
use winit::event::KeyboardInput;
use winit::event::MouseButton;
use winit::event::VirtualKeyCode::*;

use crate::renderer::RenderContext;

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
            let config = world_lock.config.as_ref().unwrap();
            let fov = config.get_var("renderer_fov").unwrap().as_f32();

            world_lock.player = Some(Player {
                transform: Transform::new(
                    Vec3::new(-20.717182, 216.95955, 98.43643),
                    Vec3::new(-0.30362973, 0.0, -2.5180235),
                ),
                camera: Camera {
                    fov,
                    size: PhysicalSize {
                        width: 1,
                        height: 1,
                    },
                    frame_count: 0,
                },
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
        let config = world.config.as_ref().unwrap();
        let sensitivity = config.get_var("game_input_mouse_sensitivity").unwrap().as_f32();
        let player = world
            .player
            .as_mut()
            .expect("ERROR: expected resource not found");
        match event {
            InputEvent::Keyboard(key_event) => self.keyboard_state.input_event(key_event),
            InputEvent::Mouse(delta) => {
                player.transform.add_rotation(Vec3::new(
                    delta.1 as f32 * -sensitivity,
                    0.0,
                    delta.0 as f32 * -sensitivity,
                )); //TODO; configuration
            }
            InputEvent::MouseButton(_) => todo!(),
        }
    }

    pub async fn update(&mut self, delta: f32) {
        let mut world = self.world.lock().await;
        let config = world.config.as_ref().unwrap();
        let move_speed = config.get_var("game_input_movement_speed").unwrap().as_f32();
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

        let mut look_delta = Vec3::new(0.0, 0.0, 0.0);

        if self.keyboard_state.is_pressed(Up) {
            look_delta.x += 1.0;
        }

        if self.keyboard_state.is_pressed(Down) {
            look_delta.x -= 1.0;
        }

        if self.keyboard_state.is_pressed(Left) {
            look_delta.z += 1.0;
        }

        if self.keyboard_state.is_pressed(Right) {
            look_delta.z -= 1.0;
        }

        look_delta = look_delta.normalize_or_zero() * delta * 1.0;

        move_dir = move_dir.normalize_or_zero() * delta * move_speed;

        player.transform.walk(move_dir);
        player.transform.add_rotation(look_delta);
    }
}
