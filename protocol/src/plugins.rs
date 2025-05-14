use std::{any::Any, sync::Arc};

use common::collections::AnyMap;
use tracing::{Instrument, Level, debug, error, span};

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
            instance.span().in_scope(|| {
                if let Some(fut) = span!(Level::INFO, "startup").in_scope(|| instance.start()) {
                    let async_span = span!(Level::INFO, "async");
                    handles.push((tokio::spawn(fut.instrument(async_span.clone())), async_span));
                }
            });
        }

        for (handle, async_span) in handles {
            match handle.await {
                Ok(()) => (),
                Err(e) => async_span.in_scope(|| error!(target: "plugins", "task panicked: {e}")),
            }
        }
    }

    pub fn shutdown(&self) {
        for plugin in self.iter().rev() {
            plugin
                .span()
                .in_scope(|| span!(Level::INFO, "shutdown").in_scope(|| plugin.shutdown()))
        }
    }

    pub fn provide<U: Any + Send + Sync + Plugin + 'static>(&mut self, instance: Arc<U>) -> Arc<U> {
        if let Some(existing) = self.instances.get::<Arc<U>>() {
            return existing.clone();
        }
        instance.span().in_scope(|| debug!("provided"));

        self.instances.insert(instance.clone());
        self.trait_objects.push(instance.clone());

        instance
    }

    pub fn load<U: Any + Send + Sync + ManagedPlugin + 'static>(&mut self) -> Arc<U> {
        if let Some(existing) = self.instances.get::<Arc<U>>() {
            return existing.clone();
        }

        let instance = U::new(self);
        instance.span().in_scope(|| debug!("loaded"));

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
