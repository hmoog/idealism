use std::{any::Any, sync::Arc};

use common::collections::AnyMap;
use protocol::Plugins;

use crate::{Config, ProtocolPlugins};

#[derive(Default)]
pub struct ProtocolParams {
    params: AnyMap,
    plugins: ProtocolPlugins,
}

impl ProtocolParams {
    pub fn with_plugins(mut self, plugins: ProtocolPlugins) -> Self {
        self.plugins = plugins;
        self
    }
}

impl protocol::ProtocolConfig for Config {
    fn with_params<T: Any + Send + Sync + 'static>(mut self, params: T) -> Self {
        self.protocol_params.params.insert(Arc::new(params));
        self
    }

    fn params<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.protocol_params.params.get::<Arc<T>>().cloned()
    }

    fn inject_plugins(&self, mut registry: Plugins) -> Plugins {
        self.protocol_params.plugins.inject(self, &mut registry);
        registry
    }
}
