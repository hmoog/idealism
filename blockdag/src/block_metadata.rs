use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
    sync::{Arc, Mutex, Weak},
};
use types::Block;
use utils::rx::{CallbackOnce, CallbacksOnce, Signal, Subscription};

use crate::{accepted::Accepted};

pub struct BlockMetadata(Arc<Inner>);

pub struct Inner {
    parents: Mutex<Vec<BlockMetadataRef>>,
    processed: Signal<()>,
    pub block: Block,
    pub accepted: Signal<Accepted>,
}

impl BlockMetadata {
    pub fn new(block: Block) -> Self {
        Self(Arc::new(Inner {
            parents: Mutex::new(vec![BlockMetadataRef::new(); block.parents().len()]),
            processed: Signal::new(),
            accepted: Signal::new(),
            block,
        }))
    }

    pub fn is_accepted(&self, chain_id: u64) -> bool {
        self.0
            .accepted
            .get()
            .as_ref()
            .is_some_and(|a| a.chain_id == chain_id)
    }

    pub fn on_processed(&self, callback: impl CallbackOnce<()>) -> Subscription<CallbacksOnce<()>> {
        self.0.processed.subscribe(callback)
    }

    pub fn downgrade(&self) -> BlockMetadataRef {
        BlockMetadataRef(Arc::downgrade(&self.0))
    }

    pub(crate) fn register_parent(&self, index: usize, parent: BlockMetadataRef) {
        self.0
            .parents
            .lock()
            .expect("failed to lock parents")
            .insert(index, parent);
    }

    pub(crate) fn mark_processed(&self) {
        self.0.processed.set(());
    }
}

impl Deref for BlockMetadata {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for BlockMetadata {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl PartialEq for BlockMetadata {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for BlockMetadata {}

impl Hash for BlockMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Arc::as_ptr(&self.0), state);
    }
}

impl Debug for BlockMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockMetadata")
            .field("block", &self.block)
            .finish()
    }
}

pub struct BlockMetadataRef(Weak<Inner>);

impl BlockMetadataRef {
    pub fn new() -> Self {
        Self(Weak::new())
    }

    pub fn upgrade(&self) -> Option<BlockMetadata> {
        self.0.upgrade().map(BlockMetadata)
    }
}

impl Default for BlockMetadataRef {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BlockMetadataRef {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl PartialEq for BlockMetadataRef {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl Eq for BlockMetadataRef {}

impl Hash for BlockMetadataRef {
    fn hash<T: Hasher>(&self, state: &mut T) {
        ptr::hash(self.0.as_ptr(), state);
    }
}
