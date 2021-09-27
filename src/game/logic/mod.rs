use glam::Vec3;

use super::{Camera, Player, Transform};

pub struct GameLogic<'a> {
    player: &'a mut Option<Player>,
}

impl<'a> GameLogic<'a> {
    pub fn new(player: &'a mut Option<Player>) -> Self {
        *player = Some(Player {
            transform: Transform::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(0.0, 0.0, 0.0)),
            camera: Camera { fov: 90.0 },
        });

        Self { player }
    }
}
