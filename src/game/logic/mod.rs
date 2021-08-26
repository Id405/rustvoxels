use cgmath::{One, Vector3};

use super::{Camera, Player, Transform};

pub struct GameLogic<'a> {
    player: &'a mut Option<Player>,
}

impl<'a> GameLogic<'a> {
    pub fn new(player: &'a mut Option<Player>) -> Self {
        *player = Some(Player {
            transform: Transform::new(
                Vector3 {
                    x: 0.0,
                    y: 10.5,
                    z: 0.0,
                },
                cgmath::Basis3::one(),
            ),
            camera: Camera { fov: 90.0 },
        });

        Self { player }
    }
}
