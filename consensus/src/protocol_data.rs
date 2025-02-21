use std::{collections::HashMap, sync::Mutex};

use blockdag::BlockDAG;
use utils::rx;
use utils::rx::Event;
use virtual_voting::{Config, Vote, Votes};

use crate::{
    block::Block,
    block_id::BlockID,
    error::{Error::VoteNotFound, Result},
};
use crate::error::Error;

pub struct ProtocolData<C: Config> {
    pub(crate) blocks: BlockDAG<Block<C>>,
    pub(crate) votes: Mutex<HashMap<BlockID, Vote<C>>>,
    pub(crate) error_event: Event<Error>,
    pub(crate) latest_accepted_milestone: rx::Variable<Vote<C>>
}

impl<C: Config> ProtocolData<C> {
    pub fn new(config: C) -> Self {
        Self {
            blocks: BlockDAG::new(),
            votes: Mutex::new(HashMap::new()),
            error_event: Event::new(),
            latest_accepted_milestone: rx::Variable::new(),
        }
    }

    pub fn votes(&self, block_ids: &[BlockID]) -> Result<Votes<C>> {
        let votes = self.votes.lock().expect("failed to lock votes");

        let mut result = Votes::default();
        for block_id in block_ids {
            result.insert(votes.get(block_id).ok_or(VoteNotFound)?.clone());
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
}
