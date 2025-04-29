use std::{ops::Deref, sync::Arc};

use common::{
    plugins::{Plugin, PluginRegistry},
    rx::Event,
};
use consensus::Consensus;
use protocol::{ProtocolConfig, ProtocolPlugin};

use crate::consensus_feed::{
    ConsensusFeedEvent,
    ConsensusFeedEvent::{
        ChainIndexUpdated, CommitteeUpdated, HeaviestMilestoneVoteUpdated,
        LatestAcceptedMilestoneUpdated,
    },
};

#[derive(Default)]
pub struct ConsensusFeed<C: ProtocolConfig> {
    event: Event<ConsensusFeedEvent<C>>,
    consensus: Arc<Consensus<C>>,
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for ConsensusFeed<C> {
    fn shutdown(&self) {
        todo!()
    }
}

impl<C: ProtocolConfig> ConsensusFeed<C> {
    fn init_plugin(self) -> Arc<Self> {
        let plugin = Arc::new(self);

        plugin.consensus.chain_index.attach({
            let plugin = plugin.clone();
            move |(old, new)| plugin.event.trigger(&ChainIndexUpdated(*old, *new))
        });

        plugin.consensus.heaviest_milestone_vote.attach({
            let plugin = plugin.clone();
            move |(old, new)| {
                plugin
                    .event
                    .trigger(&HeaviestMilestoneVoteUpdated(old.clone(), new.clone()))
            }
        });

        plugin.consensus.latest_accepted_milestone.attach({
            let plugin = plugin.clone();
            move |(old, new)| {
                plugin
                    .event
                    .trigger(&LatestAcceptedMilestoneUpdated(old.clone(), new.clone()))
            }
        });

        plugin.consensus.committee.attach({
            let plugin = plugin.clone();
            move |(old, new)| {
                plugin
                    .event
                    .trigger(&CommitteeUpdated(old.clone(), new.clone()))
            }
        });

        plugin
    }
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for ConsensusFeed<C> {
    fn construct(dependencies: &mut PluginRegistry<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Self {
            event: Default::default(),
            consensus: dependencies.load(),
        }
        .init_plugin()
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}

impl<C: ProtocolConfig> Deref for ConsensusFeed<C> {
    type Target = Event<ConsensusFeedEvent<C>>;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}
