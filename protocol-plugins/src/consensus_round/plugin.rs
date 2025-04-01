use std::{collections::HashSet, sync::Arc};

use protocol::{Protocol, ProtocolConfig, ProtocolPlugin, Result};
use types::{
    bft::Member,
    ids::IssuerID,
    plugins::{Plugin, PluginManager},
    rx::{
        UpdateType::{Notify, Retain},
        Variable,
    },
};
use virtual_voting::{Issuer, Vote};

use crate::consensus::Consensus;

#[derive(Default)]
pub struct ConsensusRound<C: ProtocolConfig> {
    pub consensus: Arc<Consensus<C>>,
    pub started: Arc<Variable<u64>>,
    pub completed: Variable<u64>,
    pub seen_participants: Arc<Variable<HashSet<IssuerID>>>,
    pub seen_weight: Arc<Variable<u64>>,
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for ConsensusRound<C> {
    fn construct(mgr: &mut PluginManager<dyn ProtocolPlugin<C>>) -> Self {
        let plugin = Self {
            started: Default::default(),
            completed: Default::default(),
            seen_participants: Default::default(),
            seen_weight: Default::default(),
            consensus: mgr.load(),
        };

        plugin
            .consensus
            .heaviest_milestone_vote
            .subscribe({
                let started = plugin.started.clone();
                let seen_participants = plugin.seen_participants.clone();
                let seen_weight = plugin.seen_weight.clone();

                move |(_, new)| {
                    let Some(new) = new else { return; };

                    started
                        .compute::<(), _>(|old| match old {
                            Some(old) if old >= new.round => Retain(Some(old)),
                            _ => {
                                seen_participants.set(HashSet::new());
                                seen_weight.set(0);

                                Notify(old, Some(new.round))
                            }
                        })
                        .expect("must not fail");
                }
            })
            .forever();

        plugin
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>>
    where
        Self: Sized,
    {
        arc
    }
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for ConsensusRound<C> {
    fn init(&self, _protocol: &Protocol<C>) {}

    fn process_vote(&self, _protocol: &Protocol<C>, vote: &Vote<C>) -> Result<()> {
        if vote.milestone.is_some() {
            match &vote.issuer {
                Issuer::User(issuer) => self.consensus.committee.must_read(|committee| {
                    if let Some(member) = committee.member(issuer) {
                        let (threshold, _does_confirm) = committee.consensus_threshold();

                        self.started.must_read(|round| {
                            if vote.round == *round {
                                self.update_seen_participants(&vote, member, threshold);
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
