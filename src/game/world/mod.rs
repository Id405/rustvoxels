mod voxel_grid;

pub use voxel_grid::VoxelGrid;

use crate::{
    config::Config,
    game::Player,
    renderer::RenderContext,
    ui::{Ui, UiState},
};

#[derive(Default)]
pub struct World {
    pub player: Option<Player>,
    pub voxel_grid: Option<VoxelGrid>,
    pub ui: Option<Ui>,
    pub config: Option<Config>,
}

impl World {
    pub fn new(context: &RenderContext) -> Self {
        let mut world = Self::default();
        world.config = Some(Config::new());
        world.voxel_grid = Some(VoxelGrid::from_string(
            std::fs::read_to_string("assets/scenes/streetcorner.evox")
                .expect("ERROR failed to load scene"),
        )); // todo config refactor
        world.voxel_grid.as_mut().unwrap().gen_texture(context);
        world.ui = Some(Ui::new(context));
        world
    }
}
