use std::{any::Any, sync::Arc};

use common::collections::AnyMap;
use tracing::{Instrument, debug};

use crate::{ManagedPlugin, Plugin};

#[derive(Default)]
pub struct Plugins {
    instances: AnyMap,
    trait_objects: Vec<Arc<dyn Plugin>>,
}

impl Plugins {
    pub async fn start(&self) {
        for plugin in self.iter() {
            plugin.start().instrument(plugin.span()).await;
        }
    }

    pub async fn shutdown(&self) {
        for plugin in self.iter().rev() {
            plugin.shutdown().instrument(plugin.span()).await;
        }
    }

    pub fn provide<U: Any + Send + Sync + Plugin + 'static>(&mut self, instance: Arc<U>) -> Arc<U> {
        if let Some(existing) = self.instances.get::<Arc<U>>() {
            return existing.clone();
        }
        instance.span().in_scope(|| debug!("plugin provided"));

        self.instances.insert(instance.clone());
        self.trait_objects.push(instance.clone());

        instance
    }

    pub fn load<U: Any + Send + Sync + ManagedPlugin + 'static>(&mut self) -> Arc<U> {
        if let Some(existing) = self.instances.get::<Arc<U>>() {
            return existing.clone();
        }

        let instance = U::new(self);
        instance.span().in_scope(|| debug!("plugin loaded"));

        self.instances.insert(instance.clone());
        self.trait_objects.push(instance.clone());

        instance
    }

    pub fn get<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.instances.get::<Arc<T>>().map(Arc::clone)
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Arc<dyn Plugin>> {
        self.trait_objects.iter()
    }
}
