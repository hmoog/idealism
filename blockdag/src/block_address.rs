use std::sync::Arc;
use types::Block;
use utils::rx::{CallbackOnce, CallbacksOnce, Signal, Subscription};

use crate::BlockMetadata;

/// BlockAddress is a helper struct that allows to publish and subscribe to
/// block metadata.
pub(crate) struct BlockAddress {
    /// Signal that holds the block metadata.
    data: Arc<Signal<BlockMetadata>>,
}

/// Implementation of BlockAddress.
impl BlockAddress {
    /// Creates a new BlockAddress.
    pub fn new() -> Self {
        Self {
            data: Arc::new(Signal::new()),
        }
    }

    pub fn data(&self) -> &Signal<BlockMetadata> {
        &self.data
    }

    /// Publishes a block.
    pub fn publish(&self, block: Block) -> BlockMetadata {
        self.data
            .get_or_insert_with(|| BlockMetadata::new(block))
            .clone()
            .unwrap()
    }

    /// Subscribes to the block metadata.
    pub fn on_available(
        &self,
        callback: impl CallbackOnce<BlockMetadata>,
    ) -> Subscription<CallbacksOnce<BlockMetadata>> {
        self.data.subscribe(callback)
    }
}

/// Clone implementation for BlockAddress.
impl Clone for BlockAddress {
    /// Clones the BlockAddress.
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}
