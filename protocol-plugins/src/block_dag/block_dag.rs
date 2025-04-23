use std::sync::{Arc, Weak};

use common::{
    blocks::BlockMetadata,
    plugins::{Plugin, PluginRegistry},
    rx::{Callbacks, Event, Subscription},
};
use protocol::{ProtocolConfig, ProtocolPlugin, ProtocolResult};

use crate::{
    block_dag::BlockDAGMetadata,
    block_storage::{Address, BlockStorage},
};

pub struct BlockDAG {
    block_storage: Arc<BlockStorage>,
    subscription: Option<Subscription<Callbacks<Address>>>,
    event: Event<BlockMetadata>,
}

impl BlockDAG {
    fn new_metadata(self: &Arc<Self>, block: &BlockMetadata) -> Arc<BlockDAGMetadata> {
        let metadata = Arc::new(BlockDAGMetadata::new(block.block.parents().len()));

        for (index, parent_id) in block.block.parents().iter().enumerate() {
            self.block_storage
                .address(parent_id)
                .subscribe({
                    let weak_metadata = Arc::downgrade(&metadata);

                    move |parent| {
                        if let Some(metadata) = weak_metadata.upgrade() {
                            metadata.set_parent_available(index, parent)
                        }
                    }
                })
                .retain();
        }

        metadata
            .pending_parents
            .subscribe({
                let block_dag = Arc::downgrade(self);
                let weak_metadata = Arc::downgrade(&metadata);
                let block = block.downgrade();

                move |_| {
                    if let Some(block_dag) = block_dag.upgrade() {
                        if let Some(block) = block.upgrade() {
                            if let Some(metadata) = weak_metadata.upgrade() {
                                block_dag.event.trigger(&block);
                                
                                metadata.available.set(());
                            }
                            
                        }
                    }
                }
            })
            .retain();

        metadata
    }

    pub fn shutdown(&mut self) {
        self.subscription.take();
    }
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for BlockDAG {
    fn construct(plugins: &mut PluginRegistry<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Arc::new_cyclic(|block_dag: &Weak<Self>| {
            let block_storage: Arc<BlockStorage> = plugins.load();

            Self {
                subscription: Some(block_storage.subscribe({
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
                })),
                block_storage,
                event: Event::default(),
            }
        })
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for BlockDAG {
    fn process_block(&self, _: &blockdag::BlockMetadata<C>) -> ProtocolResult<()> {
        Ok(())
    }
}
