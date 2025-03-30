use std::{collections::HashSet, fmt::Debug, sync::Arc};

use blockdag::{Accepted, BlockMetadata, Error::BlockNotFound};
use indexmap::IndexSet;
use types::{
    bft::{Committee, Member},
    ids::IssuerID,
    rx::{
        Event, UpdateType,
        UpdateType::{Notify, Retain},
        Variable,
    },
};
use virtual_voting::{Issuer, Vote};

use crate::{ProtocolConfig, Result};

#[derive(Default)]
pub struct State<C: ProtocolConfig> {
    pub chain_index: Variable<u64>,
    pub heaviest_milestone: Variable<Vote<C>>,
    pub latest_accepted_milestone: Variable<Vote<C>>,
    pub committee: Arc<Variable<Committee>>,
    pub round: Arc<Variable<u64>>,
    pub finalizable_round: Arc<Variable<u64>>,
    pub round_participants: Variable<HashSet<IssuerID>>,
    pub round_weight: Arc<Variable<u64>>,
    pub accepted_blocks: Event<AcceptedBlocks<C>>,
}

impl<C: ProtocolConfig> State<C> {
    pub fn analyze_vote(&self, new: &Vote<C>) {
        match &new.issuer {
            Issuer::User(issuer) => self.committee.must_read(|committee| {
                if let Some(member) = committee.member(issuer) {
                    self.round.must_read(|round| {
                        if new.round == *round {
                            self.update_round_participants(new, committee, member);
                        }
                    });

                    self.latest_accepted_milestone.must_read(|accepted| {
                        if new.round > accepted.round + 1 {
                            println!("TODO: track optimistic acceptance");
                        }
                    });
                }
            }),
            Issuer::Genesis => {
                // TODO: GENESIS
            }
        };
    }

    fn update_round_participants(&self, vote: &Vote<C>, committee: &Committee, member: &Member) {
        self.round_participants
            .compute::<(), _>(|participants| {
                let mut participants = participants.unwrap_or_default();

                if participants.insert(member.id().clone()) {
                    self.update_round_weight(
                        vote.round,
                        member.weight(),
                        committee.consensus_threshold().0,
                    );
                }

                Notify(None, Some(participants))
            })
            .expect("must not fail");
    }

    fn update_round_weight(&self, round: u64, weight: u64, threshold: u64) {
        self.round_weight
            .compute::<(), _>(|old| {
                let new = old.unwrap_or(0) + weight;
                if new > threshold {
                    self.finalizable_round.track_max(round);
                    // TODO: trigger event
                    println!("Round weight exceeded threshold {}", round);
                }

                Notify(old, Some(new))
            })
            .expect("must not fail");
    }

    pub fn init(&self, genesis: Vote<C>) {
        let derived_round = self.round.clone();
        let derived_committee = self.committee.clone();

        self.heaviest_milestone
            .subscribe(move |update| {
                if let Some(milestone) = &update.1 {
                    derived_round.track_max(milestone.round);
                    derived_committee.set_if_none_or(milestone.committee.clone(), |old, new| {
                        new.commitment() != old.commitment()
                    });
                }
            })
            .forever();

        self.heaviest_milestone.set(genesis.clone());
        self.latest_accepted_milestone.set(genesis);
    }

    pub fn apply(&self, vote: Vote<C>) -> Result<()> {
        if let Some(milestone) = &vote.milestone {
            let new = Vote::try_from(&milestone.accepted)?;
            let advance_acceptance = |old| match old {
                Some(old) if old >= new => Retain(Some(old)),
                Some(old) => match self.advance_acceptance(&old, &new) {
                    Err(err) => UpdateType::Error(Some(old), err),
                    _ => Notify(Some(old), Some(new)),
                },
                _ => Notify(old, Some(new)),
            };

            self.latest_accepted_milestone.compute(advance_acceptance)?;
            self.heaviest_milestone.track_max(vote.clone());
            self.analyze_vote(&vote);
        };

        Ok(())
    }

    fn advance_acceptance(&self, old: &Vote<C>, new: &Vote<C>) -> Result<()> {
        let height = old.height()?;
        match new.height()?.checked_sub(height) {
            None | Some(0) => panic!("TODO: implement reorg"),
            Some(range) => {
                let milestones = new.milestone_range(range)?;
                match milestones.last().expect("must exist") == old {
                    false => panic!("TODO: implement reorg"),
                    true => self
                        .accepted_blocks
                        .trigger(&self.accepted_blocks(height, milestones)?),
                }
            }
        }

        Ok(())
    }

    fn accepted_blocks(&self, height: u64, milestones: Vec<Vote<C>>) -> Result<AcceptedBlocks<C>> {
        let mut accepted_blocks = AcceptedBlocks {
            height,
            rounds: Vec::with_capacity(milestones.len()),
        };

        for (height_index, accepted_milestone) in milestones.iter().rev().enumerate() {
            let block = accepted_milestone.source.upgrade().ok_or(BlockNotFound)?;
            let past_cone = block.past_cone(|b| !b.is_accepted(0))?;

            for (round_index, block) in past_cone.iter().rev().enumerate() {
                block.accepted.set(Accepted {
                    chain_id: 0,
                    height: height + (height_index + 1) as u64,
                    round_index: round_index as u64,
                });
            }

            accepted_blocks.rounds.push(past_cone);
        }

        Ok(accepted_blocks)
    }
}

pub struct AcceptedBlocks<C: ProtocolConfig> {
    pub height: u64,
    pub rounds: Vec<IndexSet<BlockMetadata<C>>>,
}

impl<C: ProtocolConfig> Debug for AcceptedBlocks<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlocksOrderedEvent")
            .field("current_height", &self.height)
            .field("ordered_blocks", &self.rounds)
            .finish()
    }
}
