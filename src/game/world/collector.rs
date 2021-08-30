use std::path;

use crate::renderer::RenderContext;

use super::VoxelGrid;

pub struct CollectorReferences<'a> {
    pub voxel_grid: &'a mut Option<VoxelGrid>,
}

pub struct Collector;

impl Collector {
    pub fn collect(context: &RenderContext, references: CollectorReferences) {
        *references.voxel_grid = Some(VoxelGrid::from_string(
            std::fs::read_to_string("../assets/scenes/garfield.evox").expect("ERROR failed to load scene"), // Rust resources are messed up
        )); // todo config refactor
        references.voxel_grid.as_mut().unwrap().gen_texture(context);
    }
}
