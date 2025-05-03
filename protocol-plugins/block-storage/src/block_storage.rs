use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
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
    new_address: Event<Address>,
}

impl ManagedPlugin for BlockStorage {
    fn new(_: &mut Plugins) -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn start(&self) {
        self.insert(GenesisBlock(Id::default()));
    }

    fn shutdown(&self) {
        self.blocks.lock().unwrap().clear();
    }
}

impl BlockStorage {
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
            self.new_address.trigger(&address);
        }

        address
    }

    pub fn subscribe_to_new_address(
        &self,
        callback: impl Callback<Address>,
    ) -> Subscription<Callbacks<Address>> {
        self.new_address.subscribe(callback)
    }

    pub fn plugin_subscribe_new_block<Plugin: Sync + Send + 'static>(
        &self,
        weak_plugin: &Weak<Plugin>,
        callback: fn(Arc<Plugin>, &BlockMetadata),
    ) -> Subscription<Callbacks<Address>> {
        let weak_plugin = weak_plugin.clone();

        self.new_address.subscribe(move |address| {
            let weak_plugin = weak_plugin.clone();

            address.attach(move |block| {
                if let Some(plugin) = weak_plugin.upgrade() {
                    callback(plugin, block);
                }
            })
        })
    }
}
