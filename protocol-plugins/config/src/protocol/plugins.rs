use block_dag::BlockDAG;
use block_factory::BlockFactory;
use block_storage::BlockStorage;
use consensus::Consensus;
use consensus_feed::ConsensusFeed;
use consensus_round::ConsensusRound;
use inbox::Inbox;
use networking::Networking;
use outbox::Outbox;
use protocol::Plugins;
use tip_selection::TipSelection;
use virtual_voting::VirtualVoting;

use crate::Config;

pub enum ProtocolPlugins {
    Core,
    Custom(fn(&Config, &mut Plugins)),
}

impl ProtocolPlugins {
    pub fn inject(&self, config: &Config, registry: &mut Plugins) {
        match self {
            Self::Core => {
                registry.load::<BlockStorage>();
                registry.load::<BlockDAG>();
                registry.load::<VirtualVoting<Config>>();
                registry.load::<TipSelection<Config>>();
                registry.load::<Outbox>();
                registry.load::<Inbox>();
                registry.load::<Networking>();
                registry.load::<Consensus<Config>>();
                registry.load::<ConsensusRound<Config>>();
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
