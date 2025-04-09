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
use protocol::{ProtocolConfig, ProtocolPlugin, ProtocolResult};
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

impl<C: ProtocolConfig> ConsensusRound<C> {
    fn init_plugin(self) -> Arc<Self> {
        let plugin = Arc::new(self);

        plugin.consensus.heaviest_milestone_vote.attach({
            let plugin = plugin.clone();
            move |(_, new)| {
                if let Some(new) = new {
                    plugin.update_started(new.round);
                }
            }
        });

        plugin
    }

    fn process_vote(&self, vote: &Vote<C>) -> ProtocolResult<()> {
        if vote.milestone.is_none() {
            return Ok(());
        }

        self.started.must_read(|round| {
            if vote.round != *round {
                return;
            }

            self.consensus.committee.must_read(|committee| {
                let (threshold, _) = committee.consensus_threshold();

                match &vote.issuer {
                    Issuer::User(issuer) => {
                        if let Some(member) = committee.member(issuer) {
                            self.update_seen_participants(vote.round, member, threshold);
                        }
                    }
                    Issuer::Genesis => {
                        for (issuer, _) in &vote.referenced_milestones {
                            if let Some(member) = committee.member(issuer) {
                                self.update_seen_participants(vote.round, member, threshold);
                            }
                        }
                    }
                };
            });
        });

        Ok(())
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

    fn update_seen_participants(&self, round: u64, member: &Member, threshold: u64) {
        self.seen_participants
            .compute::<(), _>(|participants| {
                let mut participants = participants.unwrap_or_default();
                if participants.insert(member.id().clone()) {
                    self.update_seen_weight(round, member.weight(), threshold);
                }

                Notify(None, Some(participants))
            })
            .expect("seen_participants.compute should never fail");
    }

    fn update_seen_weight(&self, round: u64, weight: u64, threshold: u64) {
        let result = self.seen_weight.compute::<(), _>(|old| {
            let new = old.unwrap_or(0) + weight;
            if new > threshold {
                self.completed.track_max(round);
            }

            Notify(old, Some(new))
        });
        result.expect("seen_weight.compute should never fail");
    }
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for ConsensusRound<C> {
    fn construct(dependencies: &mut PluginRegistry<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Self {
            started: Default::default(),
            completed: Default::default(),
            seen_participants: Default::default(),
            seen_weight: Default::default(),
            consensus: dependencies.load(),
        }
        .init_plugin()
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for ConsensusRound<C> {
    fn process_block(&self, block: &BlockMetadata<C>) -> ProtocolResult<()> {
        self.process_vote(&block.try_get()?)
    }
}
