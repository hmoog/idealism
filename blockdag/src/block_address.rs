use std::sync::Arc;

use types::{
    blocks::Block,
    rx::{CallbackOnce, CallbacksOnce, Signal, Subscription},
};

use crate::{BlockMetadata, Config};

/// BlockAddress is a helper struct that allows to publish and subscribe to
/// block metadata.
pub(crate) struct BlockAddress<C: Config> {
    /// Signal that holds the block metadata.
    data: Arc<Signal<BlockMetadata<C>>>,
}

/// Implementation of BlockAddress.
impl<C: Config> BlockAddress<C> {
    /// Creates a new BlockAddress.
    pub fn new() -> Self {
        Self {
            data: Arc::new(Signal::new()),
        }
    }

    pub fn data(&self) -> &Signal<BlockMetadata<C>> {
        &self.data
    }

    /// Publishes a block.
    pub fn publish(&self, block: Block) -> BlockMetadata<C> {
        self.data
            .get_or_insert_with(|| BlockMetadata::new(block))
            .clone()
            .unwrap()
    }

    /// Subscribes to the block metadata.
    pub fn on_available(
        &self,
        callback: impl CallbackOnce<BlockMetadata<C>>,
    ) -> Subscription<CallbacksOnce<BlockMetadata<C>>> {
        self.data.subscribe(callback)
    }
}

/// Clone implementation for BlockAddress.
impl<C: Config> Clone for BlockAddress<C> {
    /// Clones the BlockAddress.
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}
