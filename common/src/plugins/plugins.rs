use std::{any::Any, sync::Arc};

use crate::{
    collections::AnyMap,
    plugins::{ManagedPlugin, Plugin},
};

pub struct Plugins {
    instances: AnyMap,
    trait_objects: Vec<Arc<dyn Plugin>>,
}

impl Plugins {
    pub fn provide<U: Any + Send + Sync + Plugin + 'static>(&mut self, instance: Arc<U>) -> Arc<U> {
        if let Some(existing) = self.instances.get::<Arc<U>>() {
            return existing.clone();
        }

        self.instances.insert(instance.clone());
        self.trait_objects.push(instance.clone());

        instance
    }

    pub fn load<U: Any + Send + Sync + ManagedPlugin + 'static>(&mut self) -> Arc<U> {
        if let Some(existing) = self.instances.get::<Arc<U>>() {
            return existing.clone();
        }

        let instance = U::construct(self);
        self.instances.insert(instance.clone());
        self.trait_objects.push(instance.clone());

        instance
    }

    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.instances.get::<Arc<T>>().map(Arc::clone)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn Plugin>> {
        self.trait_objects.iter()
    }
}

impl Default for Plugins {
    fn default() -> Self {
        Self {
            instances: Default::default(),
            trait_objects: Vec::new(),
        }
    }
}
