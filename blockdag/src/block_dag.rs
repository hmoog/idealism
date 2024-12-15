use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use utils::rx::{Callback, Callbacks, Countdown, Event, ResourceGuard, Subscription};
use crate::block_metadata::BlockMetadata;
use crate::block_address::BlockAddress;
use crate::traits;

pub struct BlockDAG<Block: traits::Block>(Arc<BlockDAGData<Block>>);

struct BlockDAGData<Block: traits::Block> {
    blocks: Mutex<HashMap<Block::ID, BlockAddress<Block>>>,
    ready_event: Event<ResourceGuard<BlockMetadata<Block>>>,
}

impl<Block: traits::Block> BlockDAG<Block> {
    pub fn new() -> Self {
        Self(Arc::new(BlockDAGData {
            blocks: Mutex::new(HashMap::new()),
            ready_event: Event::new(),
        }))
    }

    pub fn queue(&self, block: Block) {
        self.address(block.id().clone()).publish(block);
    }

    pub fn on_ready(&self, callback: impl Callback<ResourceGuard<BlockMetadata<Block>>>) -> Subscription<Callbacks<ResourceGuard<BlockMetadata<Block>>>> {
        self.0.ready_event.subscribe(callback)
    }

    fn address(&self, block_id: Block::ID) -> BlockAddress<Block> {
        self.0.blocks.lock().unwrap().entry(block_id.clone()).or_insert_with(|| {
            let new_address = BlockAddress::new();
            self.monitor_address(&new_address);
            new_address
        }).clone()
    }

    fn monitor_address(&self, new_address: &BlockAddress<Block>) {
        let block_dag = self.clone();

        new_address.on_available(move |block|
            block_dag.on_all_parents_processed(block, {
                let block_dag = block_dag.clone();
                let block = block.clone();

                move || block_dag.0.ready_event.trigger(
                    &ResourceGuard::new(block.clone(), BlockMetadata::mark_processed)
                )
            })
        ).forever();
    }

    fn on_all_parents_processed(&self, block: &BlockMetadata<Block>, callback: impl Fn() + Send + Sync + 'static) {
        let pending_parents = Countdown::new(block.parents().len(), callback);

        for (index, parent_id) in block.parents().iter().enumerate() {
            let block = block.clone();
            let pending_parents = pending_parents.clone();

            self.address(parent_id.clone()).on_available(move |parent| {
                block.register_parent(index, parent.downgrade());

                parent.on_processed(move |_| pending_parents.decrease()).forever()
            }).forever();
        }
    }
}

impl<Block: traits::Block> Default for BlockDAG<Block> {
    fn default() -> Self {
        Self::new()
    }
}

impl <Block: traits::Block> Clone for BlockDAG<Block> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

