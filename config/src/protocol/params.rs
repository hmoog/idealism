use common::plugins::PluginRegistry;
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

impl protocol::ProtocolConfig for Config {
    fn inject_plugins(
        &self,
        mut registry: PluginRegistry<dyn ProtocolPlugin<Self>>,
    ) -> PluginRegistry<dyn ProtocolPlugin<Self>> {
        self.protocol_params.plugins.inject(self, &mut registry);
        registry
    }
}
