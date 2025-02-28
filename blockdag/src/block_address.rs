use std::sync::Arc;

use utils::rx::{CallbackOnce, CallbacksOnce, Signal, Subscription};

use crate::{BlockMetadata, block};

/// BlockAddress is a helper struct that allows to publish and subscribe to
/// block metadata.
pub(crate) struct BlockAddress<Block: block::Block> {
    /// Signal that holds the block metadata.
    data: Arc<Signal<BlockMetadata<Block>>>,
}

/// Implementation of BlockAddress.
impl<Block: block::Block> BlockAddress<Block> {
    /// Creates a new BlockAddress.
    pub fn new() -> Self {
        Self {
            data: Arc::new(Signal::new()),
        }
    }

    pub fn data(&self) -> &Signal<BlockMetadata<Block>> {
        &self.data
    }

    /// Publishes a block.
    pub fn publish(&self, block: Block) -> BlockMetadata<Block> {
        self.data
            .get_or_insert_with(|| BlockMetadata::new(block))
            .clone()
            .unwrap()
    }

    /// Subscribes to the block metadata.
    pub fn on_available(
        &self,
        callback: impl CallbackOnce<BlockMetadata<Block>>,
    ) -> Subscription<CallbacksOnce<BlockMetadata<Block>>> {
        self.data.subscribe(callback)
    }
}

/// Clone implementation for BlockAddress.
impl<Block: block::Block> Clone for BlockAddress<Block> {
    /// Clones the BlockAddress.
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}
