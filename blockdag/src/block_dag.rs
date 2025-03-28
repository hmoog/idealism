use std::{
    collections::{
        HashMap,
        hash_map::Entry::{Occupied, Vacant},
    },
    sync::{Arc, Mutex},
};

use types::{
    blocks::Block,
    ids::BlockID,
    rx::{Callback, Callbacks, Countdown, Event, ResourceGuard, Subscription, Variable},
};
use virtual_voting::Vote;

use crate::{Config, block_address::BlockAddress, block_metadata::BlockMetadata};

pub struct BlockDAG<C: Config>(Arc<BlockDAGData<C>>);

struct BlockDAGData<C: Config> {
    genesis: Variable<BlockMetadata<C>>,
    blocks: Mutex<HashMap<BlockID, BlockAddress<C>>>,
    ready_event: Event<ResourceGuard<BlockMetadata<C>>>,
}

impl<C: Config> BlockDAG<C> {
    pub fn init(&self, genesis: Block, config: C) {
        let genesis_metadata = self.attach(genesis);
        let genesis_vote = Vote::new_genesis(genesis_metadata.downgrade(), config);
        genesis_metadata.vote.set(genesis_vote);

        self.0.genesis.set(genesis_metadata);
    }

    pub fn genesis(&self) -> BlockMetadata<C> {
        self.0
            .genesis
            .get()
            .as_ref()
            .cloned()
            .expect("genesis must be initialized")
    }

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
            .forever();
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

impl<C: Config> Default for BlockDAG<C> {
    fn default() -> Self {
        Self(Arc::new(BlockDAGData {
            genesis: Variable::new(),
            blocks: Mutex::new(HashMap::new()),
            ready_event: Event::new(),
        }))
    }
}

impl<C: Config> Clone for BlockDAG<C> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
