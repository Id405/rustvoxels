use super::VoxelGrid;

pub struct CollectorReferences<'a> {
    pub voxel_grid: &'a mut Option<VoxelGrid>,
}

pub struct Collector;

impl Collector {
    pub fn collect(references: CollectorReferences) {
        *references.voxel_grid = Some(VoxelGrid::from_string(
            std::fs::read_to_string("assets/scenes/garfield.evox")
                .expect("ERROR: failed to load scene"),
        )); // todo config refactor
    }
}
