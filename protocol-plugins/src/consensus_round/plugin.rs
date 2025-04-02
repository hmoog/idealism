use std::{collections::HashSet, sync::Arc};

use blockdag::BlockMetadata;
use common::{
    bft::Member,
    ids::IssuerID,
    plugins::{Plugin, PluginRegistry},
    rx::{
        UpdateType::{Notify, Retain},
        Variable,
    },
};
use protocol::{ProtocolConfig, ProtocolPlugin, Result};
use virtual_voting::{Issuer, Vote};

use crate::consensus::Consensus;

#[derive(Default)]
pub struct ConsensusRound<C: ProtocolConfig> {
    pub started: Variable<u64>,
    pub completed: Variable<u64>,
    pub seen_participants: Variable<HashSet<IssuerID>>,
    pub seen_weight: Variable<u64>,
    consensus: Arc<Consensus<C>>,
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for ConsensusRound<C> {
    fn process_block(&self, block: &BlockMetadata<C>) -> Result<()> {
        let vote = &block.vote()?;

        if vote.milestone.is_some() {
            match &vote.issuer {
                Issuer::User(issuer) => self.consensus.committee.must_read(|committee| {
                    if let Some(member) = committee.member(issuer) {
                        let (threshold, _does_confirm) = committee.consensus_threshold();

                        self.started.must_read(|round| {
                            if vote.round == *round {
                                self.update_seen_participants(vote, member, threshold);
                            }
                        });
                    }
                }),
                Issuer::Genesis => {
                    // TODO: GENESIS
                }
            };
        };

        Ok(())
    }
}

impl<C: ProtocolConfig> ConsensusRound<C> {
    fn init(self: Arc<Self>) -> Arc<Self> {
        self.consensus
            .heaviest_milestone_vote
            .subscribe({
                let plugin = self.clone();
                move |(_, new)| {
                    plugin.update_started(new.as_ref().unwrap().round);
                }
            })
            .forever();

        self
    }

    fn update_started(&self, new: u64) {
        self.started
            .compute::<(), _>(|old| match old {
                Some(old) if old >= new => Retain(Some(old)),
                _ => {
                    self.seen_participants.set(HashSet::new());
                    self.seen_weight.set(0);

                    Notify(old, Some(new))
                }
            })
            .expect("must not fail");
    }

    fn update_seen_participants(&self, vote: &Vote<C>, member: &Member, threshold: u64) {
        self.seen_participants
            .compute::<(), _>(|participants| {
                let mut participants = participants.unwrap_or_default();
                if participants.insert(member.id().clone()) {
                    self.update_seen_weight(vote.round, member.weight(), threshold);
                }

                Notify(None, Some(participants))
            })
            .expect("must not fail");
    }

    fn update_seen_weight(&self, round: u64, weight: u64, threshold: u64) {
        self.seen_weight
            .compute::<(), _>(|old| {
                let new = old.unwrap_or(0) + weight;
                if new > threshold {
                    self.completed.track_max(round);
                }

                Notify(old, Some(new))
            })
            .expect("must not fail");
    }
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for ConsensusRound<C> {
    fn construct(dependencies: &mut PluginRegistry<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Arc::new(Self {
            started: Default::default(),
            completed: Default::default(),
            seen_participants: Default::default(),
            seen_weight: Default::default(),
            consensus: dependencies.load(),
        })
        .init()
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}
