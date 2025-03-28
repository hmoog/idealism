use std::fmt::Debug;

use blockdag::BlockMetadata;
use indexmap::IndexSet;
use types::rx::Event;
use virtual_voting::Config;

use crate::Error;

pub struct Events<C: Config> {
    pub error: Event<Error>,
    pub blocks_ordered: Event<AcceptedBlocks<C>>,
}

impl<C: Config> Events<C> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<C: Config> Default for Events<C> {
    fn default() -> Self {
        Self {
            error: Event::new(),
            blocks_ordered: Event::new(),
        }
    }
}

pub struct AcceptedBlocks<C: Config> {
    pub height: u64,
    pub rounds: Vec<IndexSet<BlockMetadata<C>>>,
}

impl<C: Config> Debug for AcceptedBlocks<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlocksOrderedEvent")
            .field("current_height", &self.height)
            .field("ordered_blocks", &self.rounds)
            .finish()
    }
}
