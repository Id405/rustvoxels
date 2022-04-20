use std::{any::Any, sync::Arc};

use futures::lock::Mutex;

use crate::game::World;

#[derive(Clone, Copy)]
pub struct Handle {
    pub uuid: u64,
}

impl Handle {
    pub async fn with(self, world: Arc<Mutex<World>>, component: Box<dyn Any>) -> Self {
        {
            let mut world = world.lock().await;
            world
                .entities
                .get_mut(&self.uuid)
                .unwrap()
                .add_component(component);
        }

        self
    }
}
