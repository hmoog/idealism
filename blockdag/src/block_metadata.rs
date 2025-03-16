use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
    sync::{Arc, Mutex, Weak},
};

use types::blocks::Block;
use utils::rx::{CallbackOnce, CallbacksOnce, Signal, Subscription};
use virtual_voting::{Config, Vote};

use crate::accepted::Accepted;

pub struct BlockMetadata<C: Config>(Arc<Inner<C>>);

pub struct Inner<C: Config> {
    parents: Mutex<Vec<BlockMetadataRef<C>>>,
    processed: Signal<()>,
    pub block: Block,
    pub accepted: Signal<Accepted>,
    pub vote: Signal<Vote<C>>,
}

impl<C: Config> BlockMetadata<C> {
    pub fn new(block: Block) -> Self {
        Self(Arc::new(Inner {
            parents: Mutex::new(vec![BlockMetadataRef::new(); block.parents().len()]),
            processed: Signal::new(),
            block,
            accepted: Signal::new(),
            vote: Signal::new(),
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

    pub fn downgrade(&self) -> BlockMetadataRef<C> {
        BlockMetadataRef(Arc::downgrade(&self.0))
    }

    pub(crate) fn register_parent(&self, index: usize, parent: BlockMetadataRef<C>) {
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

impl<C: Config> Deref for BlockMetadata<C> {
    type Target = Inner<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Config> Clone for BlockMetadata<C> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<C: Config> PartialEq for BlockMetadata<C> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<C: Config> Eq for BlockMetadata<C> {}

impl<C: Config> Hash for BlockMetadata<C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Arc::as_ptr(&self.0), state);
    }
}

impl<C: Config> Debug for BlockMetadata<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockMetadata")
            .field("block", &self.block)
            .finish()
    }
}

pub struct BlockMetadataRef<C: Config>(Weak<Inner<C>>);

impl<C: Config> BlockMetadataRef<C> {
    pub fn new() -> Self {
        Self(Weak::new())
    }

    pub fn upgrade(&self) -> Option<BlockMetadata<C>> {
        self.0.upgrade().map(BlockMetadata)
    }
}

impl<C: Config> Default for BlockMetadataRef<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Config> Clone for BlockMetadataRef<C> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl<C: Config> PartialEq for BlockMetadataRef<C> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<C: Config> Eq for BlockMetadataRef<C> {}

impl<C: Config> Hash for BlockMetadataRef<C> {
    fn hash<T: Hasher>(&self, state: &mut T) {
        ptr::hash(self.0.as_ptr(), state);
    }
}
