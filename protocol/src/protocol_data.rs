use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use blockdag::{BlockDAG, BlockMetadata};
use indexmap::IndexSet;
use types::{blocks::Block, ids::BlockID};
use utils::rx::{Event, Variable};
use virtual_voting::{Config, Vote, Votes};

use crate::{
    error::{Error, Error::VoteNotFound, Result},
    events::BlocksOrderedEvent,
    tips::Tips,
};

pub struct ProtocolData<C: Config> {
    pub error: Event<Error>,
    pub blocks_ordered: Event<BlocksOrderedEvent<C>>,
    pub(crate) blocks: BlockDAG<C>,
    pub(crate) votes: Mutex<HashMap<BlockID, Vote<C>>>,
    pub(crate) latest_accepted_milestone: Variable<Vote<C>>,
    pub(crate) tips: Tips<C>,
}

impl<C: Config> ProtocolData<C> {
    pub fn new(config: C) -> Self {
        let genesis_vote = Vote::new_genesis(config);

        let blocks = BlockDAG::new();
        blocks.queue(Block::GenesisBlock(genesis_vote.block_id.clone()));

        Self {
            blocks,
            votes: Mutex::new(HashMap::from([(
                genesis_vote.block_id.clone(),
                genesis_vote,
            )])),
            error: Event::new(),
            latest_accepted_milestone: Variable::new(),
            blocks_ordered: Event::new(),
            tips: Tips::new(),
        }
    }

    pub fn block(&self, block_id: &BlockID) -> Option<BlockMetadata<C>> {
        self.blocks.get(block_id)
    }

    pub fn votes(&self, block_ids: &[BlockID]) -> Result<Votes<C>> {
        let locked_votes = self.votes.lock().expect("failed to lock votes");

        let mut result = Votes::default();
        for block_id in block_ids {
            result.insert(locked_votes.get(block_id).ok_or(VoteNotFound)?.clone());
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
