use std::fmt::Debug;

use blockdag::BlockMetadata;
use indexmap::IndexSet;
use virtual_voting::Config;

pub struct BlocksOrderedEvent<C: Config> {
    pub current_height: u64,
    pub ordered_blocks: Vec<IndexSet<BlockMetadata<C>>>,
}

impl<C: Config> Debug for BlocksOrderedEvent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlocksOrderedEvent")
            .field("current_height", &self.current_height)
            .field("ordered_blocks", &self.ordered_blocks)
            .finish()
    }
}
