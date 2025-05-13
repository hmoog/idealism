use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
};

use common::{
    blocks::{Block, Block::GenesisBlock, BlockMetadata},
    ids::{BlockID, Id},
    rx::{Event, Signal},
};
use protocol::{ManagedPlugin, Plugins};
use tracing::{debug, info_span, trace, Span};

use crate::Address;

pub struct BlockStorage {
    pub new_address: Event<Address>,
    blocks: Mutex<HashMap<BlockID, Address>>,
    span: Span,
}

impl ManagedPlugin for BlockStorage {
    fn new(_: &mut Plugins) -> Arc<Self> {
        Arc::new(Self {
            new_address: Default::default(),
            blocks: Default::default(),
            span: info_span!("block_storage"),
        })
    }

    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        debug!(target: "block_storage", "issuing genesis block");
        self.insert(GenesisBlock(Id::default()));
        None
    }

    fn shutdown(&self) {
        self.blocks.lock().unwrap().clear();
    }

    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl BlockStorage {
    pub fn insert(&self, block: Block) -> BlockMetadata {
        self.address(block.id())
            .get_or_insert_with(|| {
                trace!(target: "block_storage", "new block metadata stored");
                BlockMetadata::new(block)
            })
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
                    trace!(target: "block_storage", "new address allocated for block");
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
}
