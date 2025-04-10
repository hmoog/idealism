use blockdag::BlockDAGPlugin;
use common::plugins::PluginRegistry;

use crate::Config;

pub enum BlockDAGPlugins {
    Core,
    Custom(fn(&Config, &mut PluginRegistry<dyn BlockDAGPlugin<Config>>)),
}

impl BlockDAGPlugins {
    pub fn inject(
        &self,
        config: &Config,
        registry: &mut PluginRegistry<dyn BlockDAGPlugin<Config>>,
    ) {
        match self {
            Self::Core => {}
            Self::Custom(handler) => handler(config, registry),
        }
    }
}

impl Default for BlockDAGPlugins {
    fn default() -> Self {
        Self::Core
    }
}
