mod voxel_grid;
mod collector;

pub use voxel_grid::VoxelGrid;

use crate::game::Player;

#[derive(Default)]
pub struct World {
    pub player: Option<Player>,
    pub voxel_grid: Option<VoxelGrid>,
}
