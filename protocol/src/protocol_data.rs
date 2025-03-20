use std::collections::VecDeque;

use blockdag::{BlockDAG, BlockMetadata};
use indexmap::IndexSet;
use types::{
    blocks::Block,
    ids::BlockID,
    rx::{Event, Variable},
};
use virtual_voting::{Config, Vote, Votes};

use crate::{
    error::{
        Error,
        Error::{BlockNotFound, VoteNotFound},
        Result,
    },
    events::BlocksOrderedEvent,
    tips::Tips,
};

pub struct ProtocolData<C: Config> {
    pub error: Event<Error>,
    pub blocks_ordered: Event<BlocksOrderedEvent<C>>,
    pub(crate) blocks: BlockDAG<C>,
    pub(crate) latest_accepted_milestone: Variable<Vote<C>>,
    pub(crate) tips: Tips<C>,
}

impl<C: Config> ProtocolData<C> {
    pub fn new(config: C) -> Self {
        let genesis_vote = Vote::new_genesis(config);

        let blocks = BlockDAG::new();
        let genesis_metadata = blocks.queue(Block::GenesisBlock(genesis_vote.block_id.clone()));
        genesis_metadata.vote.set(genesis_vote);

        let tips = Tips::new();
        let _ = tips.register(&genesis_metadata);

        Self {
            blocks,
            error: Event::new(),
            latest_accepted_milestone: Variable::new(),
            blocks_ordered: Event::new(),
            tips,
        }
    }

    pub fn block(&self, block_id: &BlockID) -> Option<BlockMetadata<C>> {
        self.blocks.get(block_id)
    }

    pub fn referenced_votes(&self, block_metadata: &BlockMetadata<C>) -> Result<Votes<C>> {
        let mut result = Votes::default();
        for block_ref in block_metadata.parents().iter() {
            match block_ref.upgrade() {
                Some(block) => match &*block.vote.get() {
                    Some(vote) => result.insert(vote.clone()),
                    None => return Err(VoteNotFound),
                },
                None => return Err(BlockNotFound),
            };
        }

        Ok(result)
    }

    pub fn milestone_range(&self, start: &Vote<C>, amount: u64) -> Result<Vec<Vote<C>>> {
        let mut range = Vec::with_capacity(amount as usize);

        let mut current_milestone = start.clone();
        for _ in 0..amount {
            let next = Vote::try_from(current_milestone.prev_milestone()?)?;
            range.push(current_milestone);
            current_milestone = next;
        }

        Ok(range)
    }

    pub fn past_cone<F: Fn(&BlockMetadata<C>) -> bool>(
        &self,
        start: BlockMetadata<C>,
        should_visit: F,
    ) -> Result<IndexSet<BlockMetadata<C>>> {
        let mut past_cone = IndexSet::new();

        if should_visit(&start) && past_cone.insert(start.clone()) {
            let mut queue = VecDeque::from([start]);

            while let Some(current) = queue.pop_front() {
                for parent_id in current.block.parents() {
                    let parent_block = self.block(parent_id).ok_or(Error::BlockNotFound)?;

                    if should_visit(&parent_block) && past_cone.insert(parent_block.clone()) {
                        queue.push_back(parent_block);
                    }
                }
            }
        }

        Ok(past_cone)
    }
}
