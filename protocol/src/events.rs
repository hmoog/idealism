use std::fmt::Debug;

use blockdag::BlockMetadata;
use indexmap::IndexSet;

use crate::block::Block;

pub struct BlocksOrderedEvent {
    pub current_height: u64,
    pub ordered_blocks: Vec<IndexSet<BlockMetadata<Block>>>,
}

impl Debug for BlocksOrderedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlocksOrderedEvent")
            .field("current_height", &self.current_height)
            .field("ordered_blocks", &self.ordered_blocks)
            .finish()
    }
}
