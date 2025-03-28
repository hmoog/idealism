use std::sync::Arc;

use blockdag::{Accepted, Error::BlockNotFound};
use types::{
    bft::Committee,
    rx::{
        Event, UpdateType,
        UpdateType::{Notify, Retain},
        Variable,
    },
};
use virtual_voting::Vote;

use crate::{AcceptedBlocks, Config, Result};

#[derive(Default)]
pub struct State<C: Config> {
    pub chain_index: Variable<u64>,
    pub heaviest_milestone: Variable<Vote<C>>,
    pub latest_accepted_milestone: Variable<Vote<C>>,
    pub round: Arc<Variable<u64>>,
    pub committee: Arc<Variable<Committee>>,
    pub accepted_blocks: Event<AcceptedBlocks<C>>,
}

impl<C: Config> State<C> {
    pub fn init(&self, genesis: &Vote<C>) {
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
        self.latest_accepted_milestone.set(genesis.clone());
    }

    pub fn apply(&self, vote: &Vote<C>) -> Result<()> {
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
