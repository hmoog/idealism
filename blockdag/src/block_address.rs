use std::sync::Arc;
use utils::rx::{CallbackOnce, CallbacksOnce};
use utils::rx::Signal;
use utils::rx::Subscription;
use crate::{traits, BlockMetadata};

pub(crate) struct BlockAddress<Block: traits::Block> {
    data: Arc<Signal<BlockMetadata<Block>>>
}

impl<Block: traits::Block> BlockAddress<Block> {
    pub fn new() -> Self {
        Self { data: Arc::new(Signal::new()) }
    }

    pub fn publish(&self, block: Block) -> BlockMetadata<Block> {
        self.data.get_or_insert_with(|| BlockMetadata::new(block)).clone().unwrap()
    }

    pub fn on_available(&self, callback: impl CallbackOnce<BlockMetadata<Block>>) -> Subscription<CallbacksOnce<BlockMetadata<Block>>> {
        self.data.subscribe(callback)
    }
}

impl <Block: traits::Block> Clone for BlockAddress<Block> {
    fn clone(&self) -> Self {
        Self { data: Arc::clone(&self.data) }
    }
}