use std::sync::{Arc, Mutex, Weak};

use common::{
    bft::Committee,
    rx::{Callbacks, Event, Subscription},
    up, with,
};
use consensus::Consensus;
use protocol::{ManagedPlugin, Plugins};
use virtual_voting::{VirtualVotingConfig, Vote};

use crate::ConsensusFeedEvent;

pub struct ConsensusFeed<C: VirtualVotingConfig> {
    pub event: Event<ConsensusFeedEvent<C>>,
    subscriptions: Mutex<Option<Subscriptions<C>>>,
}

impl<C: VirtualVotingConfig> ManagedPlugin for ConsensusFeed<C> {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|this: &Weak<ConsensusFeed<C>>| {
            let consensus = plugins.load::<Consensus<C>>();

            Self {
                event: Default::default(),
                subscriptions: Mutex::new(Some(Subscriptions {
                    chain_index: consensus.chain_index.subscribe(
                        with!(this: move |(old, new)| up!(this: {
                            this.event.trigger(&ConsensusFeedEvent::ChainIndex(
                                *old,
                                *new,
                            ))
                        })),
                    ),
                    heaviest_milestone_vote: consensus.heaviest_milestone_vote.subscribe(
                        with!(this: move |(old, new)| up!(this: {
                            this.event.trigger(&ConsensusFeedEvent::HeaviestMilestoneVote(
                                old.clone(),
                                new.clone(),
                            ))
                        })),
                    ),
                    latest_accepted_milestone: consensus.latest_accepted_milestone.subscribe(
                        with!(this: move |(old, new)| up!(this: {
                            this.event.trigger(&ConsensusFeedEvent::LatestAcceptedMilestone(
                                old.clone(),
                                new.clone(),
                            ))
                        })),
                    ),
                    committee: consensus.committee.subscribe(
                        with!(this: move |(old, new)| up!(this: {
                            this.event.trigger(&ConsensusFeedEvent::Committee(
                                old.clone(),
                                new.clone(),
                            ))
                        })),
                    ),
                })),
            }
        })
    }

    fn shutdown(&self) {
        self.subscriptions.lock().unwrap().take();
    }
}

#[allow(dead_code)] // Subscriptions are only held to keep them alive (they act as guards)
struct Subscriptions<C: VirtualVotingConfig> {
    chain_index: U64Subscription,
    heaviest_milestone_vote: VoteSubscription<C>,
    latest_accepted_milestone: VoteSubscription<C>,
    committee: CommitteeSubscription,
}

type U64Subscription = Subscription<Callbacks<(Option<u64>, Option<u64>)>>;
type VoteSubscription<C> = Subscription<Callbacks<(Option<Vote<C>>, Option<Vote<C>>)>>;
type CommitteeSubscription = Subscription<Callbacks<(Option<Committee>, Option<Committee>)>>;
