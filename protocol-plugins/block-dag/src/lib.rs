mod metadata;

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, Weak},
};

use block_storage::{Address, BlockStorage};
use common::{
    blocks::BlockMetadata,
    errors::{Error::BlockNotFound, Result},
    plugins::{ManagedPlugin, Plugins},
    rx::{Callback, Callbacks, Event, Subscription},
};
use indexmap::IndexSet;
pub use metadata::BlockDAGMetadata;
use protocol::ProtocolPlugin;

pub struct BlockDAG {
    block_storage: Arc<BlockStorage>,
    block_storage_subscription: Mutex<Option<Subscription<Callbacks<Address>>>>,
    block_available: Event<BlockMetadata>,
}

impl ManagedPlugin<dyn ProtocolPlugin> for BlockDAG {
    fn construct(plugins: &mut Plugins<dyn ProtocolPlugin>) -> Arc<Self> {
        Arc::new_cyclic(|block_dag: &Weak<Self>| {
            let block_storage: Arc<BlockStorage> = plugins.load();

            Self {
                block_storage_subscription: Mutex::new(Some(block_storage.subscribe({
                    let block_dag = block_dag.clone();
                    move |address| {
                        address
                            .subscribe({
                                let block_dag = block_dag.clone();
                                move |block| {
                                    if let Some(block_dag) = block_dag.upgrade() {
                                        block.metadata().set(block_dag.new_metadata(block));
                                    }
                                }
                            })
                            .retain()
                    }
                }))),
                block_storage,
                block_available: Event::default(),
            }
        })
    }

    fn shutdown(&self) {
        self.block_storage_subscription.lock().unwrap().take();
    }

    fn downcast(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin> {
        arc
    }
}

impl BlockDAG {
    pub fn subscribe(
        &self,
        callback: impl Callback<BlockMetadata>,
    ) -> Subscription<Callbacks<BlockMetadata>> {
        self.block_available.subscribe(callback)
    }

    pub fn past_cone<F: Fn(&BlockMetadata) -> Result<bool>>(
        start: &BlockMetadata,
        should_visit: F,
    ) -> Result<IndexSet<BlockMetadata>> {
        let mut past_cone = IndexSet::new();

        if should_visit(start)? && past_cone.insert(start.clone()) {
            let mut queue = VecDeque::from([start.clone()]);

            while let Some(current) = queue.pop_front() {
                for parent_ref in current
                    .try_get::<Arc<BlockDAGMetadata>>()?
                    .parents
                    .lock()
                    .unwrap()
                    .iter()
                {
                    let parent_block = parent_ref.upgrade().ok_or(BlockNotFound)?;

                    if should_visit(&parent_block)? && past_cone.insert(parent_block.clone()) {
                        queue.push_back(parent_block);
                    }
                }
            }
        }

        Ok(past_cone)
    }

    fn new_metadata(self: &Arc<Self>, block: &BlockMetadata) -> Arc<BlockDAGMetadata> {
        let metadata = Arc::new(BlockDAGMetadata::new(block.block.parents().len()));

        for (index, parent_id) in block.block.parents().iter().enumerate() {
            self.block_storage.address(parent_id).attach({
                let weak_metadata = Arc::downgrade(&metadata);

                move |parent| {
                    if let Some(metadata) = weak_metadata.upgrade() {
                        metadata.set_parent_available(index, parent)
                    }
                }
            });
        }

        metadata.all_parents_available.attach({
            let block_dag = Arc::downgrade(self);
            let block = block.downgrade();

            move |_| {
                if let Some(block_dag) = block_dag.upgrade() {
                    if let Some(block) = block.upgrade() {
                        block_dag.block_available.trigger(&block);
                    }
                }
            }
        });

        metadata
    }
}

impl ProtocolPlugin for BlockDAG {
    fn shutdown(&self) {
        self.block_storage_subscription.lock().unwrap().take();
    }
}
