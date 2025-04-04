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
    rx::{Countdown, Event, ResourceGuard, Signal},
};
use zero::{Clone0, Deref0};

use crate::{BlockDAGConfig, block_metadata::BlockMetadata};

#[derive(Default, Deref0, Clone0)]
pub struct BlockDAG<C: BlockDAGConfig>(Arc<BlockDAGData<C>>);

#[derive(Default)]
pub struct BlockDAGData<C: BlockDAGConfig> {
    pub block_ready: Event<ResourceGuard<BlockMetadata<C>>>,
    blocks: Mutex<HashMap<BlockID, Arc<Signal<BlockMetadata<C>>>>>,
}

impl<C: BlockDAGConfig> BlockDAG<C> {
    pub fn attach(&self, block: Block) -> BlockMetadata<C> {
        self.address(block.id())
            .get_or_insert_with(|| BlockMetadata::new(block))
            .clone()
            .unwrap()
    }

    pub fn get(&self, block_id: &BlockID) -> Option<BlockMetadata<C>> {
        let addresses = self.0.blocks.lock().unwrap();
        addresses
            .get(block_id)
            .and_then(|a| a.get().as_ref().cloned())
    }

    fn address(&self, block_id: &BlockID) -> Arc<Signal<BlockMetadata<C>>> {
        let (block_address, is_new) = match self.0.blocks.lock().unwrap().entry(block_id.clone()) {
            Occupied(entry) => (entry.get().clone(), false),
            Vacant(entry) => {
                let addr = Arc::new(Signal::default());
                entry.insert(addr.clone());
                (addr, true)
            }
        };

        if is_new {
            self.monitor_address(&block_address);
        }

        block_address
    }

    fn monitor_address(&self, new_address: &Signal<BlockMetadata<C>>) {
        let block_dag = self.clone();

        new_address
            .subscribe(move |block| {
                block_dag.on_all_parents_processed(block, {
                    let block_dag = block_dag.clone();
                    let block = block.clone();

                    move || {
                        block_dag.0.block_ready.trigger(&ResourceGuard::new(
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

            let sub = self.address(parent_id).subscribe(move |parent| {
                block.register_parent(index, parent.downgrade());

                let sub = parent.on_processed(move |_| pending_parents.decrease());
                sub.retain()
            });
            sub.retain();
        }
    }
}
