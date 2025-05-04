use std::fmt::Debug;

use common::blocks::BlockMetadata;
use indexmap::IndexSet;

pub struct AcceptedBlocks {
    pub height: u64,
    pub rounds: Vec<IndexSet<BlockMetadata>>,
}

impl Debug for AcceptedBlocks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AcceptedBlocks")
            .field("current_height", &self.height)
            .field("ordered_blocks", &self.rounds)
            .finish()
    }
}
