use block_dag::BlockDAG;
use block_storage::BlockStorage;
use common::plugins::PluginRegistry;
use consensus::Consensus;
use consensus_round::ConsensusRound;
use protocol::ProtocolPlugin;
use protocol_plugins::{
    block_factory::BlockFactory, consensus_feed::ConsensusFeed, tip_selection::TipSelection,
};
use virtual_voting::VirtualVoting;

use crate::Config;

pub enum ProtocolPlugins {
    Core,
    Custom(fn(&Config, &mut PluginRegistry<dyn ProtocolPlugin>)),
}

impl ProtocolPlugins {
    pub fn inject(&self, config: &Config, registry: &mut PluginRegistry<dyn ProtocolPlugin>) {
        match self {
            Self::Core => {
                registry.load::<BlockStorage>();
                registry.load::<BlockDAG>();
                registry.load::<VirtualVoting<Config>>();
                registry.load::<Consensus<Config>>();
                registry.load::<ConsensusRound<Config>>();
                registry.load::<TipSelection<Config>>();
                registry.load::<BlockFactory<Config>>();
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
