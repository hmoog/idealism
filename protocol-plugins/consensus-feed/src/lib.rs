use std::{ops::Deref, sync::Arc};

use common::{blocks::BlockMetadataRef, rx::Event};
use consensus::Consensus;
use virtual_voting::VirtualVotingConfig;

use crate::ConsensusFeedEvent::{
    ChainIndexUpdated, CommitteeUpdated, HeaviestMilestoneVoteUpdated,
    LatestAcceptedMilestoneUpdated,
};

#[derive(Default)]
pub struct ConsensusFeed<C: VirtualVotingConfig<Source = BlockMetadataRef>> {
    event: Event<ConsensusFeedEvent<C>>,
    consensus: Arc<Consensus<C>>,
}

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> ConsensusFeed<C> {
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

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> ManagedPlugin for ConsensusFeed<C> {
    fn construct(dependencies: &mut Plugins) -> Arc<Self> {
        Self {
            event: Default::default(),
            consensus: dependencies.load(),
        }
        .init_plugin()
    }

    fn shutdown(&self) {
        todo!()
    }
}

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> Deref for ConsensusFeed<C> {
    type Target = Event<ConsensusFeedEvent<C>>;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

mod event;

pub use event::ConsensusFeedEvent;
use protocol::{ManagedPlugin, Plugins};
