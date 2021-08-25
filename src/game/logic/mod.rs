use cgmath::{Rotation3, Vector3};

use super::{Camera, Player, Transform};

pub struct GameLogic<'a> {
    player: &'a mut Option<Player>,
}

impl<'a> GameLogic<'a> {
    fn new(player: &'a mut Option<Player>) -> Self {
        *player = Some(Player {
            transform: Transform::new(Vector3 { x: 0.0, y: 10.5, z: 0.0 }, cgmath::Basis3::from_angle_x(0.0)),
            camera: Camera { fov: 90.0 },
        });

        Self {
            player
        }
    }
}