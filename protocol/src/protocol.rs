use std::sync::Arc;

use blockdag::{Accepted, BlockMetadata, Error::BlockNotFound};
use indexmap::IndexSet;
use types::{
    blocks::{Block, NetworkBlock},
    ids::IssuerID,
    rx::{Callback, Callbacks, ResourceGuard, Subscription},
};
use virtual_voting::{Config, Vote};
use zero::{Clone0, Deref0};

use crate::{
    error::{Error, Result},
    events::BlocksOrderedEvent,
    protocol_data::ProtocolData,
};

#[derive(Deref0, Clone0)]
pub struct Protocol<C: Config>(Arc<ProtocolData<C>>);

impl<C: Config> Protocol<C> {
    pub fn new(config: C) -> Self {
        let protocol = Self(Arc::new(ProtocolData::new(config)));

        protocol
            .on_block_ready({
                let protocol = protocol.clone();

                move |block_metadata| {
                    if let Err(err) = protocol.process_block(block_metadata) {
                        protocol.error.trigger(&err);
                    }
                }
            })
            .forever();

        protocol
    }

    pub fn on_block_ready(
        &self,
        callback: impl Callback<ResourceGuard<BlockMetadata<C>>>,
    ) -> Subscription<Callbacks<ResourceGuard<BlockMetadata<C>>>> {
        self.blocks.on_block_ready(callback)
    }

    pub fn issue_block(&self, issuer: &IssuerID) {
        self.blocks.queue(Block::from(NetworkBlock {
            parents: self.tips.get(),
            issuer_id: issuer.clone(),
        }));
    }

    fn process_block(&self, metadata: &ResourceGuard<BlockMetadata<C>>) -> Result<()> {
        match &metadata.block {
            Block::NetworkBlock(id, network_block) => {
                let vote = Vote::new(
                    id.clone(),
                    &network_block.issuer_id,
                    0,
                    metadata.referenced_votes()?,
                )?;

                if let Some(milestone) = &vote.milestone {
                    self.process_milestone(Vote::try_from(&milestone.accepted)?)?;
                }

                metadata.vote.set(vote);

                self.tips.register(metadata)?;

                Ok(())
            }
            _ => Err(Error::UnsupportedBlockType),
        }
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
                        let accepted_milestones = new.milestone_range(accepted_height)?;
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
    ) -> Result<Vec<IndexSet<BlockMetadata<C>>>> {
        let mut accepted_blocks = Vec::with_capacity(accepted_milestones.len());

        for (height_index, accepted_milestone) in accepted_milestones.iter().rev().enumerate() {
            let milestone_block = self
                .blocks
                .get(&accepted_milestone.block_id)
                .ok_or(Error::BlockDagErr(BlockNotFound))?;
            let past_cone = milestone_block.past_cone(|b| !b.is_accepted(0))?;

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
