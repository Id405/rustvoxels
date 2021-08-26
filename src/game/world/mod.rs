mod collector;
mod voxel_grid;

pub use collector::Collector;
pub use collector::CollectorReferences;
pub use voxel_grid::VoxelGrid;

use crate::game::Player;

#[derive(Default)]
pub struct World {
    pub player: Option<Player>,
    pub voxel_grid: Option<VoxelGrid>,
}
