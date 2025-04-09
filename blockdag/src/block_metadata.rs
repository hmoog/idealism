use std::{
    any::TypeId,
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard, RwLock},
};

use common::{
    blocks::Block,
    collections::AnyMap,
    rx::{CallbackOnce, CallbacksOnce, Countdown, Signal, Subscription},
};
use indexmap::IndexSet;
use virtual_voting::{Vote, Votes};

use crate::{
    BlockDAGConfig, BlockMetadataRef,
    Error::{BlockNotFound, MetadataNotFound},
    accepted::Accepted,
    error::Result,
};

pub struct BlockMetadata<C: BlockDAGConfig>(pub(crate) Arc<Inner<C>>);

pub struct Inner<C: BlockDAGConfig> {
    signals: RwLock<AnyMap>,
    pub all_parents_processed: Arc<Countdown>,
    parents: Mutex<Vec<BlockMetadataRef<C>>>,
    processed: Signal<()>,
    pub block: Block,
    pub accepted: Signal<Accepted>,
    pub error: Signal<C::ErrorType>,
}

impl<C: BlockDAGConfig> BlockMetadata<C> {
    pub fn new(block: Block) -> Self {
        Self(Arc::new(Inner {
            signals: RwLock::new(AnyMap::default()),
            parents: Mutex::new(vec![BlockMetadataRef::default(); block.parents().len()]),
            all_parents_processed: Arc::new(Countdown::new(block.parents().len())),
            processed: Signal::default(),
            block,
            accepted: Signal::default(),
            error: Signal::default(),
        }))
    }

    pub fn signal<T: Send + Sync + 'static>(&self) -> Arc<Signal<T>> {
        if let Some(signal) = self.signals.read().unwrap().get::<Arc<Signal<T>>>() {
            return signal.clone();
        }

        self.signals
            .write()
            .unwrap()
            .get_or_insert_with::<Arc<Signal<T>>, _>(Arc::default)
            .clone()
    }

    pub fn try_get<T: Send + Sync + Clone + 'static>(&self) -> Result<T> {
        self.signal::<T>()
            .value()
            .ok_or(MetadataNotFound(TypeId::of::<T>()))
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
                Some(block) => result.insert(block.try_get::<Vote<C>>()?),
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

    pub(crate) fn register_parent(&self, index: usize, parent: &BlockMetadata<C>) {
        self.parents()[index] = parent.downgrade();

        parent
            .on_processed({
                let all_parents_processed = self.all_parents_processed.clone();
                move |_| all_parents_processed.decrease()
            })
            .retain();
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

    use crate::{BlockDAGConfig, BlockMetadata, Inner};

    impl<C: BlockDAGConfig> Deref for BlockMetadata<C> {
        type Target = Inner<C>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<C: BlockDAGConfig> Clone for BlockMetadata<C> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }

    impl<C: BlockDAGConfig> PartialEq for BlockMetadata<C> {
        fn eq(&self, other: &Self) -> bool {
            Arc::ptr_eq(&self.0, &other.0)
        }
    }

    impl<C: BlockDAGConfig> Eq for BlockMetadata<C> {}

    impl<C: BlockDAGConfig> Hash for BlockMetadata<C> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            ptr::hash(Arc::as_ptr(&self.0), state);
        }
    }

    impl<C: BlockDAGConfig> Debug for BlockMetadata<C> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BlockMetadata")
                .field("block", &self.block)
                .finish()
        }
    }
}
