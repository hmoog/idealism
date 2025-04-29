use block_storage::BlockStorage;
use common::plugins::PluginRegistry;
use consensus::Consensus;
use protocol::ProtocolPlugin;
use protocol_plugins::{
    block_factory::BlockFactory, consensus_feed::ConsensusFeed, consensus_round::ConsensusRound,
    tip_selection::TipSelection,
};

use crate::Config;

pub enum ProtocolPlugins {
    Core,
    Custom(fn(&Config, &mut PluginRegistry<dyn ProtocolPlugin<Config>>)),
}

impl ProtocolPlugins {
    pub fn inject(
        &self,
        config: &Config,
        registry: &mut PluginRegistry<dyn ProtocolPlugin<Config>>,
    ) {
        match self {
            Self::Core => {
                registry.load::<BlockStorage>();
                registry.load::<Consensus<Config>>();
                registry.load::<ConsensusRound<Config>>();
                registry.load::<TipSelection>();
                registry.load::<BlockFactory>();
                registry.load::<ConsensusFeed<Config>>();
            }
            Self::Custom(handler) => handler(config, registry),
        }
    }
}

impl Default for ProtocolPlugins {
    fn default() -> Self {
        Self::Core
    }
}
