use std::fmt::Debug;

use blockdag::BlockMetadata;
use indexmap::IndexSet;
use types::rx::Event;
use virtual_voting::Config;

use crate::Error;

pub struct Events<C: Config> {
    pub error: Event<Error>,
    pub blocks_ordered: Event<BlocksOrdered<C>>,
}

impl<C: Config> Events<C> {
    pub fn new() -> Self {
        Self {
            error: Event::new(),
            blocks_ordered: Event::new(),
        }
    }
}

pub struct BlocksOrdered<C: Config> {
    pub current_height: u64,
    pub ordered_blocks: Vec<IndexSet<BlockMetadata<C>>>,
}

impl<C: Config> Debug for BlocksOrdered<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlocksOrderedEvent")
            .field("current_height", &self.current_height)
            .field("ordered_blocks", &self.ordered_blocks)
            .finish()
    }
}
