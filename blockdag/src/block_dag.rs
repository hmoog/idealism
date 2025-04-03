use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    sync::{Arc, Mutex},
};

use common::{
    blocks::Block,
    ids::BlockID,
    rx::{Callback, Callbacks, Countdown, Event, ResourceGuard, Subscription},
};

use crate::{BlockDAGConfig, block_address::BlockAddress, block_metadata::BlockMetadata};

pub struct BlockDAG<C: BlockDAGConfig>(Arc<BlockDAGData<C>>);

struct BlockDAGData<C: BlockDAGConfig> {
    blocks: Mutex<HashMap<BlockID, BlockAddress<C>>>,
    ready_event: Event<ResourceGuard<BlockMetadata<C>>>,
}

impl<C: BlockDAGConfig> BlockDAG<C> {
    pub fn attach(&self, block: Block) -> BlockMetadata<C> {
        self.address(block.id()).publish(block)
    }

    pub fn on_block_ready(
        &self,
        callback: impl Callback<ResourceGuard<BlockMetadata<C>>>,
    ) -> Subscription<Callbacks<ResourceGuard<BlockMetadata<C>>>> {
        self.0.ready_event.subscribe(callback)
    }

    pub fn get(&self, block_id: &BlockID) -> Option<BlockMetadata<C>> {
        let addresses = self.0.blocks.lock().unwrap();
        addresses
            .get(block_id)
            .and_then(|a| a.data().get().as_ref().cloned())
    }

    fn address(&self, block_id: &BlockID) -> BlockAddress<C> {
        let (block_address, is_new) = match self.0.blocks.lock().unwrap().entry(block_id.clone()) {
            Occupied(entry) => (entry.get().clone(), false),
            Vacant(entry) => {
                let addr = BlockAddress::new();
                entry.insert(addr.clone());
                (addr, true)
            }
        };

        if is_new {
            self.monitor_address(&block_address);
        }

        block_address
    }

    fn monitor_address(&self, new_address: &BlockAddress<C>) {
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
            .retain();
    }

    fn on_all_parents_processed(
        &self,
        metadata: &BlockMetadata<C>,
        callback: impl Fn() + Send + Sync + 'static,
    ) {
        let parents = metadata.block.parents();
        let pending_parents = Countdown::new(parents.len(), callback);

        for (index, parent_id) in parents.iter().enumerate() {
            let block = metadata.clone();
            let pending_parents = pending_parents.clone();

            let sub = self.address(parent_id).on_available(move |parent| {
                block.register_parent(index, parent.downgrade());

                let sub = parent.on_processed(move |_| pending_parents.decrease());
                sub.retain()
            });
            sub.retain();
        }
    }
}

impl<C: BlockDAGConfig> Default for BlockDAG<C> {
    fn default() -> Self {
        Self(Arc::new(BlockDAGData {
            blocks: Mutex::new(HashMap::new()),
            ready_event: Event::new(),
        }))
    }
}

impl<C: BlockDAGConfig> Clone for BlockDAG<C> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
