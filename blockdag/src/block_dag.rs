use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use std::ops::Deref;
use common::{
    blocks::Block,
    ids::BlockID,
    rx::{Event, ResourceGuard, Signal},
};
use zero::{Clone0, Deref0};

use crate::{BlockDAGConfig, block_metadata::BlockMetadata};

#[derive(Default, Deref0, Clone0)]
pub struct BlockDAG<C: BlockDAGConfig>(Arc<BlockDAGData<C>>);

#[derive(Default)]
pub struct BlockDAGData<C: BlockDAGConfig> {
    blocks: Mutex<HashMap<BlockID, Arc<Signal<BlockMetadata<C>>>>>,
    block_ready: Event<ResourceGuard<BlockMetadata<C>>>,
}

impl<C: BlockDAGConfig> BlockDAG<C> {
    pub fn queue(&self, block: Block) -> BlockMetadata<C> {
        self.metadata_signal(block.id())
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

    fn metadata_signal(&self, block_id: &BlockID) -> Arc<Signal<BlockMetadata<C>>> {
        let mut is_new = false;

        let signal = {
            let mut blocks = self.0.blocks.lock().unwrap();
            blocks
                .entry(block_id.clone())
                .or_insert_with(|| {
                    is_new = true;
                    Arc::new(Signal::default())
                })
                .clone()
        };

        if is_new {
            signal
                .subscribe({
                    let block_dag = self.clone();
                    move |metadata| {
                        block_dag.setup_metadata(metadata);
                    }
                }).retain();
        }

        signal
    }

    fn setup_metadata(&self, metadata: &BlockMetadata<C>) {
        for (index, parent_id) in metadata.block.parents().iter().enumerate() {
            self
                .metadata_signal(parent_id)
                .subscribe({
                    let metadata = metadata.clone();
                    move |parent| {
                        metadata.register_parent(index, parent);
                    }
                })
                .retain();
        }

        metadata
            .all_parents_processed
            .subscribe({
                let block_dag = self.clone();
                let metadata = metadata.clone();
                move |_| {
                    block_dag.block_ready.trigger(&ResourceGuard::new(
                        metadata,
                        BlockMetadata::mark_processed,
                    ))
                }
            })
            .retain();
    }
}

impl<C: BlockDAGConfig> Deref for BlockDAGData<C> {
    type Target = Event<ResourceGuard<BlockMetadata<C>>>;

    fn deref(&self) -> &Self::Target {
        &self.block_ready
    }
}
