use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard},
};

use indexmap::IndexSet;
use types::{
    blocks::Block,
    rx::{CallbackOnce, CallbacksOnce, Signal, Subscription},
};
use virtual_voting::{Vote, Votes};

use crate::{
    BlockMetadataRef, Config,
    Error::{BlockNotFound, VoteNotFound},
    accepted::Accepted,
    error::Result,
};

pub struct BlockMetadata<C: Config>(pub(crate) Arc<Inner<C>>);

pub struct Inner<C: Config> {
    parents: Mutex<Vec<BlockMetadataRef<C>>>,
    processed: Signal<()>,
    pub block: Block,
    pub accepted: Signal<Accepted>,
    pub vote: Signal<Vote<C>>,
    pub error: Signal<C::ErrorType>,
}

impl<C: Config> BlockMetadata<C> {
    pub fn new(block: Block) -> Self {
        Self(Arc::new(Inner {
            parents: Mutex::new(vec![BlockMetadataRef::default(); block.parents().len()]),
            processed: Signal::new(),
            block,
            accepted: Signal::new(),
            vote: Signal::new(),
            error: Signal::new(),
        }))
    }

    pub fn vote(&self) -> Result<Vote<C>> {
        self.vote.get().as_ref().cloned().ok_or(VoteNotFound)
    }

    pub fn parents(&self) -> MutexGuard<Vec<BlockMetadataRef<C>>> {
        self.0.parents.lock().expect("failed to lock parents")
    }

    pub fn past_cone<F: Fn(&BlockMetadata<C>) -> bool>(
        &self,
        should_visit: F,
    ) -> Result<IndexSet<BlockMetadata<C>>> {
        let mut past_cone = IndexSet::new();

        if should_visit(self) && past_cone.insert(self.clone()) {
            let mut queue = VecDeque::from([self.clone()]);

            while let Some(current) = queue.pop_front() {
                for parent_ref in current.parents().iter() {
                    let parent_block = parent_ref.upgrade().ok_or(BlockNotFound)?;

                    if should_visit(&parent_block) && past_cone.insert(parent_block.clone()) {
                        queue.push_back(parent_block);
                    }
                }
            }
        }

        Ok(past_cone)
    }

    pub fn referenced_votes(&self) -> Result<Votes<C>> {
        let mut result = Votes::default();
        for block_ref in self.parents().iter() {
            match block_ref.upgrade() {
                Some(block) => match &*block.vote.get() {
                    Some(vote) => result.insert(vote.clone()),
                    None => return Err(VoteNotFound),
                },
                None => return Err(BlockNotFound),
            };
        }

        Ok(result)
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
        let mut parents = self.parents();
        parents[index] = parent;
    }

    pub(crate) fn mark_processed(&self) {
        self.0.processed.set(());
    }
}

mod traits {
    use std::{
        fmt::Debug,
        hash::{Hash, Hasher},
        ops::Deref,
        ptr,
        sync::Arc,
    };

    use crate::{BlockMetadata, Config, Inner};

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
}
