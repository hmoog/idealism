use common::plugins::{Plugin, Plugins};

use crate::{Config, ProtocolPlugins};

#[derive(Default)]
pub struct ProtocolParams {
    plugins: ProtocolPlugins,
}

impl ProtocolParams {
    pub fn with_plugins(mut self, plugins: ProtocolPlugins) -> Self {
        self.plugins = plugins;
        self
    }
}

impl protocol::ProtocolConfig for Config {
    fn inject_plugins(&self, mut registry: Plugins) -> Plugins {
        self.protocol_params.plugins.inject(self, &mut registry);
        registry
    }
}

impl Plugin for Config {
    fn shutdown(&self) {
        // do nothing
    }
}
