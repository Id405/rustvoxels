use crate::game::Player;
use super::voxel_grid::VoxelGrid;

struct GlobalState {
    player: Player,
    voxel_grid: VoxelGrid,
}