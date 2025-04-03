use common::plugins::PluginRegistry;
use protocol::ProtocolPlugin;

use crate::Config;

impl protocol::ProtocolConfig for Config {
    fn inject_plugins(
        &self,
        mut registry: PluginRegistry<dyn ProtocolPlugin<Self>>,
    ) -> PluginRegistry<dyn ProtocolPlugin<Self>> {
        self.protocol_params.plugins.inject(self, &mut registry);
        registry
    }
}
