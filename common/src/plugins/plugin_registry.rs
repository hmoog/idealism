use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use crate::plugins::Plugin;

pub struct PluginRegistry<Trait: ?Sized + 'static> {
    instances: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    trait_objects: Vec<Arc<Trait>>,
}

impl<Trait: ?Sized + 'static> PluginRegistry<Trait> {
    pub fn load<U: Any + Send + Sync + Plugin<Trait> + 'static>(&mut self) -> Arc<U> {
        let type_id = TypeId::of::<U>();

        if let Some(existing) = self.instances.get(&type_id) {
            return existing.clone().downcast_arc::<U>().unwrap();
        }

        let instance = U::construct(self);
        self.instances
            .insert(type_id, instance.clone() as Arc<dyn Any + Send + Sync>);

        let trait_object = U::plugin(instance.clone());
        self.trait_objects.push(trait_object);

        instance
    }

    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        self.instances
            .get(&type_id)
            .and_then(|arc| arc.clone().downcast_arc::<T>())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Arc<Trait>> {
        self.trait_objects.iter()
    }
}

impl<Trait: ?Sized + 'static> Default for PluginRegistry<Trait> {
    fn default() -> Self {
        Self {
            instances: HashMap::new(),
            trait_objects: Vec::new(),
        }
    }
}

pub trait DowncastArc {
    fn downcast_arc<T: Any + Send + Sync>(self: Arc<Self>) -> Option<Arc<T>>;
}

impl DowncastArc for dyn Any + Send + Sync {
    fn downcast_arc<T: Any + Send + Sync>(self: Arc<Self>) -> Option<Arc<T>> {
        if (*self).type_id() == TypeId::of::<T>() {
            let raw = Arc::into_raw(self) as *const T;
            Some(unsafe { Arc::from_raw(raw) })
        } else {
            None
        }
    }
}
