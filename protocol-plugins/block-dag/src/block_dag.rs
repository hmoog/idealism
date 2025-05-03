use std::sync::{Arc, Mutex, Weak};

use block_storage::{Address, BlockStorage};
use common::{
    blocks::BlockMetadata,
    rx::{Callback, Callbacks, Event, Subscription},
};
use protocol::{ManagedPlugin, Plugins};

use crate::BlockDAGMetadata;

pub struct BlockDAG {
    block_available: Event<BlockMetadata>,
    block_storage_subscription: Mutex<Option<Subscription<Callbacks<Address>>>>,
    block_storage: Arc<BlockStorage>,
}

impl ManagedPlugin for BlockDAG {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|this: &Weak<Self>| {
            let block_storage = plugins.load::<BlockStorage>();

            Self {
                block_available: Event::default(),
                block_storage_subscription: Mutex::new(Some(
                    block_storage.plugin_subscribe_new_block(this, |this, block| {
                        this.setup_metadata(block);
                    }),
                )),
                block_storage,
            }
        })
    }

    fn shutdown(&self) {
        self.block_storage_subscription.lock().unwrap().take();
    }
}

impl BlockDAG {
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

impl BlockDAG {
    pub fn subscribe_block_available(
        &self,
        callback: impl Callback<BlockMetadata>,
    ) -> Subscription<Callbacks<BlockMetadata>> {
        self.block_available.subscribe(callback)
    }

    pub fn plugin_subscribe_block_available<T: Sync + Send + 'static>(
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

    pub fn plugin_subscribe_metadata_available<Plugin, Metadata>(
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

    pub fn plugin_subscribe_block_and_metadata_available<Plugin, Metadata>(
        &self,
        weak_plugin: &Weak<Plugin>,
        callback: fn(Arc<Plugin>, &BlockMetadata, &Metadata),
    ) -> Subscription<Callbacks<BlockMetadata>>
    where
        Plugin: Sync + Send + 'static,
        Metadata: Clone + Send + Sync + 'static,
    {
        let weak_plugin = weak_plugin.clone();

        self.block_available.subscribe(move |block| {
            let weak_plugin = weak_plugin.clone();
            let weak_block = block.downgrade();

            block.attach(move |metadata: &Metadata| {
                if let Some(plugin) = weak_plugin.upgrade() {
                    if let Some(block) = weak_block.upgrade() {
                        callback(plugin, &block, metadata);
                    }
                }
            })
        })
    }
}
