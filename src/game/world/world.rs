use crate::game::Player;
use super::voxel_grid::VoxelGrid;

#[derive(Default)]
pub struct World {
    pub player: Option<Player>,
    pub voxel_grid: Option<VoxelGrid>,
}