pub mod components;
mod handle;

use std::any::Any;

pub use handle::Handle;

pub struct Entity {
    pub uuid: u64,
    pub components: Vec<Box<dyn Any>>,
}

impl Entity {
    pub fn new(uuid: u64) -> Self {
        Self {
            uuid,
            components: Vec::new(),
        }
    }

    pub fn add_component(&mut self, component: Box<dyn Any>) {
        self.components.push(component);
    }

    pub fn get_components<T: 'static>(&self) -> Vec<&T> {
        let mut components = Vec::new();

        for component in &self.components {
            match component.as_ref().downcast_ref::<T>() {
                Some(value) => {
                    components.push(value);
                }
                None => (),
            }
        }

        components
    }

    pub fn get_components_mut<T: 'static>(&mut self) -> Vec<&mut T> {
        let mut components = Vec::new();

        for component in &mut self.components {
            match component.as_mut().downcast_mut::<T>() {
                Some(value) => {
                    components.push(value);
                }
                None => (),
            }
        }

        components
    }
}
