mod voxel_grid;

pub use voxel_grid::VoxelGrid;

use crate::{game::Player, renderer::RenderContext};

#[derive(Default)]
pub struct World {
    pub player: Option<Player>,
    pub voxel_grid: Option<VoxelGrid>,
}

impl World {
    pub fn new(context: &RenderContext) -> Self {
        let mut world = Self::default();
        world.voxel_grid = Some(VoxelGrid::from_string(
            std::fs::read_to_string("assets/scenes/streetcorner.evox")
                .expect("ERROR failed to load scene"),
        )); // todo config refactor
        world.voxel_grid.as_mut().unwrap().gen_texture(context);
        world
    }
}
