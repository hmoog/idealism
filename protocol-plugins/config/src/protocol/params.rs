use std::sync::Arc;

use common::plugins::{ManagedPlugin, Plugin, Plugins};
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
        mut registry: Plugins<dyn ProtocolPlugin>,
    ) -> Plugins<dyn ProtocolPlugin> {
        self.protocol_params.plugins.inject(self, &mut registry);
        registry
    }
}

impl Plugin<dyn ProtocolPlugin> for Config {
    fn shutdown(&self) {
        // do nothing
    }

    fn downcast(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin> {
        arc
    }
}
