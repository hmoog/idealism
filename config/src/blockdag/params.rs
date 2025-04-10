use blockdag::BlockDAGPlugin;
use common::{ids::BlockID, plugins::PluginRegistry};

use crate::{BlockDAGPlugins, Config};

#[derive(Default)]
pub struct BlockDAGParams {
    genesis_block_id: BlockID,
    plugins: BlockDAGPlugins,
}

impl blockdag::BlockDAGConfig for Config {
    type ErrorType = protocol::ProtocolError;

    fn inject_plugins(
        &self,
        mut registry: PluginRegistry<dyn BlockDAGPlugin<Self>>,
    ) -> PluginRegistry<dyn BlockDAGPlugin<Self>> {
        self.blockdag_params.plugins.inject(self, &mut registry);
        registry
    }

    fn genesis_block_id(&self) -> BlockID {
        self.blockdag_params.genesis_block_id.clone()
    }
}
