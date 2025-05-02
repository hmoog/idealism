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
    block_storage_subscription: Mutex<Option<Subscription<Callbacks<Address>>>>,
    block_storage: Arc<BlockStorage>,
}

impl ManagedPlugin for BlockDAG {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|weak: &Weak<Self>| {
            let block_storage: Arc<BlockStorage> = plugins.load();
            let block_storage_subscription =
                block_storage.subscribe_plugin_to_new_block(weak, |this, block| {
                    this.setup_metadata(block);
                });

            Self {
                block_available: Event::default(),
                block_storage_subscription: Mutex::new(Some(block_storage_subscription)),
                block_storage,
            }
        })
    }

    fn shutdown(&self) {
        self.block_storage_subscription.lock().unwrap().take();
    }
}

impl BlockDAG {
    pub fn subscribe_plugin_to_block<T: Sync + Send + 'static>(
        &self,
        weak_plugin: &Weak<T>,
        callback: fn(Arc<T>, &BlockMetadata),
    ) -> Subscription<Callbacks<BlockMetadata>> {
        let weak_plugin = weak_plugin.clone();

        self.block_available.subscribe(move |block| {
            if let Some(plugin) = weak_plugin.upgrade() {
                callback(plugin, block);
            }
        })
    }

    pub fn subscribe_plugin_to_metadata<Plugin, Metadata>(
        &self,
        weak_plugin: &Weak<Plugin>,
        callback: fn(Arc<Plugin>, &Metadata),
    ) -> Subscription<Callbacks<BlockMetadata>>
    where
        Plugin: Sync + Send + 'static,
        Metadata: Clone + Send + Sync + 'static,
    {
        let weak_plugin = weak_plugin.clone();

        self.block_available.subscribe(move |block| {
            let weak_plugin = weak_plugin.clone();

            block.attach(move |metadata: &Metadata| {
                if let Some(plugin) = weak_plugin.upgrade() {
                    callback(plugin, metadata);
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
