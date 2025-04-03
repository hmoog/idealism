use common::plugins::PluginRegistry;
use protocol::ProtocolPlugin;
use protocol_plugins::{
    block_factory::BlockFactory, consensus::Consensus, consensus_round::ConsensusRound,
    tip_selection::TipSelection,
};

use crate::Config;

pub enum PluginLoader {
    Core,
    Custom(fn(&Config, &mut PluginRegistry<dyn ProtocolPlugin<Config>>)),
}

impl PluginLoader {
    pub fn dispatch(
        &self,
        config: &Config,
        registry: &mut PluginRegistry<dyn ProtocolPlugin<Config>>,
    ) {
        match self {
            Self::Core => {
                registry.load::<Consensus<Config>>();
                registry.load::<ConsensusRound<Config>>();
                registry.load::<TipSelection<Config>>();
                registry.load::<BlockFactory<Config>>();
            }
            Self::Custom(handler) => handler(config, registry),
        }
    }
}
