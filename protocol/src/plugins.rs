use std::{any::Any, sync::Arc};

use common::collections::AnyMap;
use tracing::{Instrument, Level, span};

use crate::{ManagedPlugin, Plugin};

#[derive(Default)]
pub struct Plugins {
    instances: AnyMap,
    trait_objects: Vec<Arc<dyn Plugin>>,
}

impl Plugins {
    pub async fn start(&self) {
        let mut handles = Vec::new();

        for instance in self.iter() {
            let plugin_span = span!(Level::INFO, "plugin", name = instance.plugin_name()).entered();

            if let Some(fut) = span!(Level::INFO, "startup").in_scope(|| instance.start()) {
                handles.push((
                    tokio::spawn(
                        fut.instrument(span!(parent: plugin_span.clone(), Level::INFO, "async")),
                    ),
                    instance.plugin_name(),
                ));
            }
        }

        for (handle, plugin_name) in handles {
            match handle.await {
                Ok(()) => (),
                Err(e) => eprintln!("async worker of plugin {plugin_name} panicked: {e}"),
            }
        }
    }

    pub fn shutdown(&self) {
        for plugin in self.iter() {
            let _plugin_span = span!(Level::INFO, "plugin", name = plugin.plugin_name()).entered();
            span!(Level::INFO, "shutdown").in_scope(|| plugin.shutdown());
        }
    }

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

        let instance = U::new(self);
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
