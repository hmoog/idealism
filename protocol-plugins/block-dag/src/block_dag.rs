use std::sync::{Arc, Mutex, Weak};

use block_storage::{Address, BlockStorage};
use common::{
    blocks::BlockMetadata,
    rx::{Callbacks, Event, Subscription},
};
use protocol::{ManagedPlugin, Plugins};

use crate::BlockDAGMetadata;

pub struct BlockDAG {
    pub block_available: Event<BlockMetadata>,
    block_storage: Arc<BlockStorage>,
    block_storage_subscription: Mutex<Option<Subscription<Callbacks<Address>>>>,
}

impl BlockDAG {
    fn new(weak: &Weak<Self>, plugins: &mut Plugins) -> Self {
        Self {
            block_available: Event::default(),
            block_storage: plugins.load(),
            block_storage_subscription: Mutex::new(Some(Self::subscribe_block_storage(
                weak.clone(),
                &plugins.load(),
            ))),
        }
    }

    fn shutdown(&self) {
        self.block_storage_subscription.lock().unwrap().take();
    }

    fn subscribe_block_storage(
        weak_block_dag: Weak<Self>,
        block_storage: &BlockStorage,
    ) -> Subscription<Callbacks<Address>> {
        block_storage.subscribe(move |address| {
            let weak_block_dag = weak_block_dag.clone();
            address.attach(move |block| {
                if let Some(block_dag) = weak_block_dag.upgrade() {
                    block_dag.setup_metadata(block);
                }
            })
        })
    }

    fn setup_metadata(self: &Arc<Self>, block: &BlockMetadata) {
        let metadata = block.set(Arc::new(BlockDAGMetadata::new(block.block.parents().len())));

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
            let weak_block_dag = Arc::downgrade(self);
            let weak_block = block.downgrade();
            move |_| {
                if let Some(block_dag) = weak_block_dag.upgrade() {
                    if let Some(block) = weak_block.upgrade() {
                        block_dag.block_available.trigger(&block);
                    }
                }
            }
        });
    }
}

impl ManagedPlugin for BlockDAG {
    fn construct(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|weak: &Weak<Self>| Self::new(weak, plugins))
    }

    fn shutdown(&self) {
        self.shutdown()
    }
}
