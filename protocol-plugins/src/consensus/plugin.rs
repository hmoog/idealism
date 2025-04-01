use std::sync::Arc;

use blockdag::{Accepted, Error::BlockNotFound};
use protocol::{Protocol, ProtocolConfig, ProtocolPlugin, Result};
use types::{
    bft::Committee,
    plugins::{Plugin, PluginManager},
    rx::{
        Event, UpdateType,
        UpdateType::{Notify, Retain},
        Variable,
    },
};
use virtual_voting::Vote;

use crate::consensus::AcceptedBlocks;

#[derive(Default)]
pub struct Consensus<C: ProtocolConfig> {
    pub chain_index: Variable<u64>,
    pub heaviest_milestone_vote: Variable<Vote<C>>,
    pub latest_accepted_milestone: Variable<Vote<C>>,
    pub committee: Variable<Committee>,
    pub accepted_blocks: Event<AcceptedBlocks<C>>,
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for Consensus<C> {
    fn construct(_manager: &mut PluginManager<dyn ProtocolPlugin<C>>) -> Self {
        Self::default()
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>>
    where
        Self: Sized,
    {
        arc
    }
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for Consensus<C> {
    fn init(&self, protocol: &Protocol<C>) {
        self.process_vote(
            protocol,
            &protocol.block_dag.genesis().vote().expect("must exist"),
        )
        .expect("must not fail");
    }

    fn process_vote(&self, _protocol: &Protocol<C>, vote: &Vote<C>) -> Result<()> {
        if vote.milestone.is_some() {
            self.update_heaviest_milestone_vote(vote)?;
            self.update_latest_accepted_milestone(vote)?;
        };

        Ok(())
    }
}

impl<C: ProtocolConfig> Consensus<C> {
    fn update_heaviest_milestone_vote(&self, vote: &Vote<C>) -> Result<()> {
        self.heaviest_milestone_vote.compute(|old| {
            let result = match old {
                Some(old) if old >= *vote => Retain(Some(old)),
                Some(old) => Notify(Some(old), Some(vote.clone())),
                _ => Notify(old, Some(vote.clone())),
            };

            if let Notify(_, Some(new)) = &result {
                self.committee
                    .set_if_none_or(new.committee.clone(), |old, new| {
                        new.commitment() != old.commitment()
                    });
            }

            result
        })
    }

    fn update_latest_accepted_milestone(&self, vote: &Vote<C>) -> Result<()> {
        let new = Vote::try_from(&vote.milestone()?.accepted)?;

        self.latest_accepted_milestone.compute(|old| match old {
            Some(old) if old >= new => Retain(Some(old)),
            Some(old) => match self.advance_acceptance(&old, &new) {
                Err(err) => UpdateType::Error(Some(old), err),
                _ => Notify(Some(old), Some(new)),
            },
            _ => Notify(old, Some(new)),
        })
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
