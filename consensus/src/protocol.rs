use std::sync::Arc;

use blockdag::{Accepted, Block as _, BlockMetadata};
use indexmap::IndexSet;
use utils::rx::ResourceGuard;
use virtual_voting::{Config, Vote};
use zero::{Clone0, Deref0};

use crate::{
    error::{Error, Result},
    events::BlocksOrderedEvent,
    protocol_data::ProtocolData,
    types::Block,
};

#[derive(Deref0, Clone0)]
pub struct Protocol<C: Config>(Arc<ProtocolData<C>>);

impl<C: Config> Protocol<C> {
    pub fn new(config: C) -> Self {
        Self(Arc::new(ProtocolData::new(config)))
    }

    pub fn run(&mut self) {
        let this = self.clone();

        let process_block = move |b: &ResourceGuard<BlockMetadata<Block<C>>>| {
            let _ = this.process_block(b.block()).inspect_err(|err| this.error_event.trigger(err));
        };

        self.blocks.on_ready(process_block).forever();
    }

    fn process_block(&self, block: &Block<C>) -> Result<()> {
        let vote = Vote::new(
            block.id().clone(),
            block.issuer_id(),
            0,
            self.votes(block.parents())?,
        )?;

        self.process_vote(block, vote)
    }

    fn process_vote(&self, block: &Block<C>, vote: Vote<C>) -> Result<()> {
        if let Some(milestone) = &vote.milestone {
            self.process_milestone(Vote::try_from(&milestone.accepted)?)?;
        }

        self.votes.lock().unwrap().insert(block.id().clone(), vote);

        Ok(())
    }

    fn process_milestone(&self, new: Vote<C>) -> Result<()> {
        let mut guard = self.latest_accepted_milestone.get();
        *guard = Some('update: {
            if let Some(current) = guard.take() {
                if current >= new {
                    break 'update current;
                }

                let current_height = current.height()?;
                let new_height = new.height()?;

                match new_height.checked_sub(current_height) {
                    None | Some(0) => self.process_reorg(),
                    Some(accepted_height) => {
                        let accepted_milestones = self.milestone_range(&new, accepted_height)?;
                        match *accepted_milestones.last().expect("must exist") == current {
                            false => self.process_reorg(),
                            true => self.blocks_ordered.trigger(&BlocksOrderedEvent {
                                current_height,
                                ordered_blocks: self
                                    .advance_acceptance(current_height, accepted_milestones)?,
                            }),
                        }
                    }
                }
            }

            new
        });

        Ok(())
    }

    fn process_reorg(&self) {}

    fn advance_acceptance(
        &self,
        current_height: u64,
        accepted_milestones: Vec<Vote<C>>,
    ) -> Result<Vec<IndexSet<BlockMetadata<Block<C>>>>> {
        let mut accepted_blocks = Vec::with_capacity(accepted_milestones.len());

        for (height_index, accepted_milestone) in accepted_milestones.iter().rev().enumerate() {
            let milestone_block = self
                .block(&accepted_milestone.block_id)
                .ok_or(Error::BlockNotFound)?;
            let past_cone = self.past_cone(milestone_block, |b| !b.is_accepted(0))?;

            for (round_index, block) in past_cone.iter().rev().enumerate() {
                block.accepted.set(Accepted {
                    chain_id: 0,
                    height: current_height + (height_index + 1) as u64,
                    round_index: round_index as u64,
                });
            }

            accepted_blocks.push(past_cone);
        }

        Ok(accepted_blocks)
    }
}
