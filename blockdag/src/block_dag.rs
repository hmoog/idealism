use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use utils::rx::{Callback, Callbacks, Countdown, Event, ResourceGuard, Subscription};

use crate::{block, block_address::BlockAddress, block_metadata::BlockMetadata};

pub struct BlockDAG<Block: block::Block>(Arc<BlockDAGData<Block>>);

struct BlockDAGData<Block: block::Block> {
    blocks: Mutex<HashMap<Block::ID, BlockAddress<Block>>>,
    ready_event: Event<ResourceGuard<BlockMetadata<Block>>>,
}

impl<Block: block::Block> BlockDAG<Block> {
    pub fn new() -> Self {
        Self(Arc::new(BlockDAGData {
            blocks: Mutex::new(HashMap::new()),
            ready_event: Event::new(),
        }))
    }

    pub fn queue(&self, block: Block) {
        self.address(block.id()).publish(block);
    }

    pub fn on_ready(
        &self,
        callback: impl Callback<ResourceGuard<BlockMetadata<Block>>>,
    ) -> Subscription<Callbacks<ResourceGuard<BlockMetadata<Block>>>> {
        self.0.ready_event.subscribe(callback)
    }

    pub fn get(&self, block_id: &Block::ID) -> Option<BlockMetadata<Block>> {
        self.0
            .blocks
            .lock()
            .unwrap()
            .get(block_id)
            .and_then(|a| a.data().get().as_ref().cloned())
    }

    fn address(&self, block_id: &Block::ID) -> BlockAddress<Block> {
        self.0
            .blocks
            .lock()
            .unwrap()
            .entry(block_id.clone())
            .or_insert_with(|| {
                let new_address = BlockAddress::new();
                self.monitor_address(&new_address);
                new_address
            })
            .clone()
    }

    fn monitor_address(&self, new_address: &BlockAddress<Block>) {
        let block_dag = self.clone();

        new_address
            .on_available(move |block| {
                block_dag.on_all_parents_processed(block, {
                    let block_dag = block_dag.clone();
                    let block = block.clone();

                    move || {
                        block_dag.0.ready_event.trigger(&ResourceGuard::new(
                            block.clone(),
                            BlockMetadata::mark_processed,
                        ))
                    }
                })
            })
            .forever();
    }

    fn on_all_parents_processed(
        &self,
        metadata: &BlockMetadata<Block>,
        callback: impl Fn() + Send + Sync + 'static,
    ) {
        let parents = metadata.block().parents();
        let pending_parents = Countdown::new(parents.len(), callback);

        for (index, parent_id) in parents.iter().enumerate() {
            let block = metadata.clone();
            let pending_parents = pending_parents.clone();

            self.address(parent_id)
                .on_available(move |parent| {
                    block.register_parent(index, parent.downgrade());

                    parent
                        .on_processed(move |_| pending_parents.decrease())
                        .forever()
                })
                .forever();
        }
    }
}

impl<Block: block::Block> Default for BlockDAG<Block> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Block: block::Block> Clone for BlockDAG<Block> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
