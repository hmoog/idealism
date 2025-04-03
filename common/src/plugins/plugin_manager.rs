use std::{
    any::Any,
    sync::{Arc, RwLock},
};

use zero::Deref0;

use crate::plugins::{Plugin, PluginRegistry};

#[derive(Deref0)]
pub struct PluginManager<Trait: ?Sized + 'static>(Arc<RwLock<PluginRegistry<Trait>>>);

impl<Trait: ?Sized + 'static> PluginManager<Trait> {
    pub fn load<U: Any + Send + Sync + Plugin<Trait> + 'static>(&self) -> Arc<U> {
        let mut plugins = self.0.write().unwrap();
        plugins.load::<U>()
    }

    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let plugins = self.0.read().unwrap();
        plugins.get::<T>()
    }

    pub fn for_each<E>(&self, mut f: impl FnMut(&Trait) -> Result<(), E>) -> Result<(), E> {
        for plugin in self.0.read().unwrap().iter() {
            f(plugin)?;
        }

        Ok(())
    }
}

impl<Trait: ?Sized + 'static> Default for PluginManager<Trait> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(PluginRegistry::default())))
    }
}
