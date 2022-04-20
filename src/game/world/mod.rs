use std::{
    any::{Any, TypeId},
    collections::HashMap,
    convert::TryInto,
    sync::Arc,
    time::Instant,
};

use futures::lock::Mutex;

use crate::{
    config::Config,
    game::Player,
    renderer::RenderContext,
    ui::{Ui, UiState},
};

use super::{entity::Handle, Entity};

#[derive(Default)]
pub struct World {
    pub entity_count: u64,
    pub entities: HashMap<u64, Entity>,

    pub player: Option<Player>, // These need to go bye bye
    pub ui: Option<Ui>,
    pub config: Option<Config>,
}

impl World {
    pub fn new(context: &RenderContext) -> Self {
        let mut world = Self::default();
        world.config = Some(Config::new());
        world.ui = Some(Ui::new(context));

        world
    }

    pub fn create_entity(&mut self) -> Handle {
        self.entity_count += 1;
        self.entities
            .insert(self.entity_count, Entity::new(self.entity_count));
        Handle {
            uuid: self.entity_count,
        }
    }

    pub fn add_component(&mut self, handle: Handle, component: Box<dyn Any>) {
        self.entities
            .get_mut(&handle.uuid)
            .unwrap()
            .add_component(component);
    }

    pub fn get_entities<T: 'static>(&self) -> Vec<&Entity> {
        let mut vec = Vec::new();

        for (_, entity) in self.entities.iter() {
            for component in &entity.components {
                match component.as_ref().downcast_ref::<T>() {
                    Some(value) => {
                        vec.push(entity);
                        break;
                    }
                    None => (),
                }
            }
        }

        vec
    }

    pub fn get_entities_mut<T: 'static>(&mut self) -> Vec<&mut Entity> {
        let mut vec = Vec::new();

        for (_, entity) in self.entities.iter_mut() {
            for component in &mut entity.components {
                match component.as_ref().downcast_ref::<T>() {
                    Some(value) => {
                        vec.push(entity);
                        break;
                    }
                    None => (),
                }
            }
        }

        vec
    }

    pub fn get_components<T: 'static>(&self) -> Vec<&T> {
        let mut components = Vec::new();

        for (_, entity) in self.entities.iter() {
            for component in &entity.components {
                match component.as_ref().downcast_ref::<T>() {
                    Some(value) => {
                        components.push(value);
                    }
                    None => (),
                }
            }
        }

        components
    }

    pub fn get_components_mut<T: 'static>(&mut self) -> Vec<&mut T> {
        let mut components = Vec::new();

        for (_, entity) in self.entities.iter_mut() {
            for component in &mut entity.components {
                match component.as_mut().downcast_mut::<T>() {
                    Some(value) => {
                        components.push(value);
                    }
                    None => (),
                }
            }
        }

        components
    }
}
