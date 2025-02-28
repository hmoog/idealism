use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
    sync::{Arc, Mutex, Weak},
};

use utils::rx::{CallbackOnce, CallbacksOnce, Signal, Subscription};

use crate::{accepted::Accepted, block};

pub struct BlockMetadata<Block: block::Block>(Arc<Inner<Block>>);

pub struct Inner<Block: block::Block> {
    parents: Mutex<Vec<BlockMetadataRef<Block>>>,
    processed: Signal<()>,
    pub accepted: Signal<Accepted>,
    block: Arc<Block>,
}

impl<Block: block::Block> BlockMetadata<Block> {
    pub fn new(block: Block) -> Self {
        Self(Arc::new(Inner {
            parents: Mutex::new(vec![BlockMetadataRef::new(); block.parents().len()]),
            processed: Signal::new(),
            accepted: Signal::new(),
            block: Arc::new(block),
        }))
    }

    pub fn block(&self) -> &Block {
        &self.0.block
    }

    pub fn is_accepted(&self, chain_id: u64) -> bool {
        self.0
            .accepted
            .get()
            .as_ref()
            .map_or(false, |a| a.chain_id == chain_id)
    }

    pub fn on_processed(&self, callback: impl CallbackOnce<()>) -> Subscription<CallbacksOnce<()>> {
        self.0.processed.subscribe(callback)
    }

    pub fn downgrade(&self) -> BlockMetadataRef<Block> {
        BlockMetadataRef(Arc::downgrade(&self.0))
    }

    pub(crate) fn register_parent(&self, index: usize, parent: BlockMetadataRef<Block>) {
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

impl<Block: block::Block> Deref for BlockMetadata<Block> {
    type Target = Inner<Block>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Block: block::Block> Clone for BlockMetadata<Block> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<Block: block::Block> PartialEq for BlockMetadata<Block> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<Block: block::Block> Eq for BlockMetadata<Block> {}

impl<Block: block::Block> Hash for BlockMetadata<Block> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Arc::as_ptr(&self.0), state);
    }
}

impl<Block: block::Block> Debug for BlockMetadata<Block> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockMetadata")
            .field("block", self.block())
            .finish()
    }
}

pub struct BlockMetadataRef<Block: block::Block>(Weak<Inner<Block>>);

impl<Block: block::Block> BlockMetadataRef<Block> {
    pub fn new() -> Self {
        Self(Weak::new())
    }

    pub fn upgrade(&self) -> Option<BlockMetadata<Block>> {
        self.0.upgrade().map(BlockMetadata)
    }
}

impl<Block: block::Block> Default for BlockMetadataRef<Block> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Block: block::Block> Clone for BlockMetadataRef<Block> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl<Block: block::Block> PartialEq for BlockMetadataRef<Block> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<Block: block::Block> Eq for BlockMetadataRef<Block> {}

impl<Block: block::Block> Hash for BlockMetadataRef<Block> {
    fn hash<T: Hasher>(&self, state: &mut T) {
        ptr::hash(self.0.as_ptr(), state);
    }
}
