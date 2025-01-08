use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};
use utils::rx::{CallbackOnce, CallbacksOnce, Signal, Subscription};
use crate::block;

pub struct BlockMetadata<Block: block::Block>(Arc<Inner<Block>>);

struct Inner<Block: block::Block> {
    parents: Mutex<Vec<BlockMetadataRef<Block>>>,
    processed: Signal<()>,
    block: Arc<Block>,
}

impl<Block: block::Block> BlockMetadata<Block> {
    pub fn new(block: Block) -> Self {
        Self(Arc::new(Inner {
            parents: Mutex::new(vec![BlockMetadataRef::new(); block.parents().len()]),
            processed: Signal::new(),
            block: Arc::new(block),
        }))
    }

    pub fn block(&self) -> &Block {
        &self.0.block
    }

    pub fn on_processed(&self, callback: impl CallbackOnce<()>) -> Subscription<CallbacksOnce<()>> {
        self.0.processed.subscribe(callback)
    }

    pub fn downgrade(&self) -> BlockMetadataRef<Block> {
        BlockMetadataRef(Arc::downgrade(&self.0))
    }

    pub(crate) fn register_parent(&self, index: usize, parent: BlockMetadataRef<Block>) {
        self.0.parents.lock()
            .expect("failed to lock parents")
            .insert(index, parent);
    }

    pub(crate) fn mark_processed(&self) {
        self.0.processed.set(());
    }
}

impl<Block: block::Block> Deref for BlockMetadata<Block> {
    type Target = Block;

    fn deref(&self) -> &Self::Target {
        self.block()
    }
}

impl <Block: block::Block> Clone for BlockMetadata<Block> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
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