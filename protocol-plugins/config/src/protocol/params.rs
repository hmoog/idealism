use std::sync::Arc;

use common::plugins::{Plugin, PluginRegistry};
use protocol::ProtocolPlugin;

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

impl ProtocolPlugin for Config {
    fn shutdown(&self) {
        todo!()
    }
}

impl protocol::ProtocolConfig for Config {
    fn inject_plugins(
        &self,
        mut registry: PluginRegistry<dyn ProtocolPlugin>,
    ) -> PluginRegistry<dyn ProtocolPlugin> {
        self.protocol_params.plugins.inject(self, &mut registry);
        registry
    }
}

impl Plugin<dyn ProtocolPlugin> for Config {
    fn construct(_: &mut PluginRegistry<dyn ProtocolPlugin>) -> Arc<Self> {
        panic!("Config should not be constructed automatically");
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin> {
        arc
    }
}
