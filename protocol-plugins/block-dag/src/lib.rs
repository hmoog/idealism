mod block_metadata_ext;
mod metadata;

use std::sync::{Arc, Mutex, Weak};

pub use block_metadata_ext::BlockDAGBlockMetadataExt;
use block_storage::{Address, BlockStorage};
use common::{
    blocks::BlockMetadata,
    rx::{Callback, Callbacks, Event, Subscription},
};
pub use metadata::BlockDAGMetadata;
use protocol::{ManagedPlugin, Plugins};

pub struct BlockDAG {
    block_storage: Arc<BlockStorage>,
    block_storage_subscription: Mutex<Option<BlockStorageSubscription>>,
    block_available: Event<BlockMetadata>,
}

impl BlockDAG {
    fn new(weak: &Weak<Self>, plugins: &mut Plugins) -> Self {
        let block_storage: Arc<BlockStorage> = plugins.load();

        Self {
            block_storage_subscription: Mutex::new(Some(Self::block_storage_subscription(
                &block_storage,
                weak.clone(),
            ))),
            block_storage,
            block_available: Event::default(),
        }
    }

    fn shutdown(&self) {
        self.block_storage_subscription.lock().unwrap().take();
    }

    fn block_storage_subscription(
        block_storage: &Arc<BlockStorage>,
        weak: Weak<Self>,
    ) -> BlockStorageSubscription {
        block_storage.subscribe(move |address| {
            let weak = weak.clone();
            address.attach(move |block| {
                if let Some(block_dag) = weak.upgrade() {
                    block_dag.init_metadata(block);
                }
            })
        })
    }

    fn init_metadata(self: &Arc<Self>, block: &BlockMetadata) -> Arc<BlockDAGMetadata> {
        let metadata = Arc::new(BlockDAGMetadata::new(block.block.parents().len()));
        block.metadata().set(metadata.clone());

        for (index, parent_id) in block.block.parents().iter().enumerate() {
            self.block_storage.address(parent_id).attach({
                let weak_metadata = Arc::downgrade(&metadata);

                move |parent| {
                    if let Some(metadata) = weak_metadata.upgrade() {
                        println!("parent available");
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

impl BlockDAG {
    pub fn subscribe(&self, callback: impl Callback<BlockMetadata>) -> BlockDAGSubscription {
        self.block_available.subscribe(callback)
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

type BlockStorageSubscription = Subscription<Callbacks<Address>>;
type BlockDAGSubscription = Subscription<Callbacks<BlockMetadata>>;
