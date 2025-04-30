use std::{any::Any, sync::Arc};

use crate::{collections::AnyMap, plugins::ManagedPlugin};
use crate::plugins::Plugin;

pub struct Plugins<Trait: ?Sized + 'static> {
    instances: AnyMap,
    trait_objects: Vec<Arc<Trait>>,
}

impl<Trait: ?Sized + 'static> Plugins<Trait> {
    pub fn provide<U: Any + Send + Sync + Plugin<Trait> + 'static>(
        &mut self,
        instance: Arc<U>,
    ) -> Arc<U> {
        if let Some(existing) = self.instances.get::<Arc<U>>() {
            return existing.clone();
        }

        self.instances.insert(instance.clone());

        let trait_object = U::downcast(instance.clone());
        self.trait_objects.push(trait_object);

        instance
    }

    pub fn load<U: Any + Send + Sync + ManagedPlugin<Trait> + 'static>(&mut self) -> Arc<U> {
        if let Some(existing) = self.instances.get::<Arc<U>>() {
            return existing.clone();
        }

        let instance = U::construct(self);
        self.instances.insert(instance.clone());

        let trait_object = U::downcast(instance.clone());
        self.trait_objects.push(trait_object);

        instance
    }

    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.instances.get::<Arc<T>>().map(Arc::clone)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<Trait>> {
        self.trait_objects.iter()
    }
}

impl<Trait: ?Sized + 'static> Default for Plugins<Trait> {
    fn default() -> Self {
        Self {
            instances: Default::default(),
            trait_objects: Vec::new(),
        }
    }
}
