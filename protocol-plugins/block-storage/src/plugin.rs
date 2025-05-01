use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use common::{
    blocks::{Block, Block::GenesisBlock, BlockMetadata},
    ids::{BlockID, Id},
    rx::{Callback, Callbacks, Event, Signal, Subscription},
};
use protocol::{ManagedPlugin, Plugins};

use crate::Address;

#[derive(Default)]
pub struct BlockStorage {
    blocks: Mutex<HashMap<BlockID, Address>>,
    notification: Event<Address>,
}

impl BlockStorage {
    fn start(&self) {
        println!("BlockStorage started");

        self.insert(GenesisBlock(Id::default()));
    }

    pub fn insert(&self, block: Block) -> BlockMetadata {
        self.address(block.id())
            .get_or_insert_with(|| BlockMetadata::new(block))
            .clone()
            .unwrap()
    }

    pub fn get(&self, block_id: &BlockID) -> Option<BlockMetadata> {
        let addresses = self.blocks.lock().unwrap();
        addresses
            .get(block_id)
            .and_then(|a| a.get().as_ref().cloned())
    }

    pub fn address(&self, block_id: &BlockID) -> Address {
        let mut is_new = false;

        let address = {
            let mut blocks = self.blocks.lock().unwrap();
            blocks
                .entry(block_id.clone())
                .or_insert_with(|| {
                    is_new = true;
                    Arc::new(Signal::default())
                })
                .clone()
        };

        if is_new {
            self.notification.trigger(&address);
        }

        address
    }

    pub fn subscribe(&self, callback: impl Callback<Address>) -> Subscription<Callbacks<Address>> {
        self.notification.subscribe(callback)
    }
}

impl ManagedPlugin for BlockStorage {
    fn construct(_manager: &mut Plugins) -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn start(&self) {
        self.start();
    }

    fn shutdown(&self) {
        self.blocks.lock().unwrap().clear();
    }
}
