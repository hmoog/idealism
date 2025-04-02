use std::{fmt, fmt::Debug};

use crate::{blocks::NetworkBlock, ids::BlockID};

pub enum Block {
    GenesisBlock(BlockID),
    NetworkBlock(BlockID, NetworkBlock),
}

impl Block {
    pub fn id(&self) -> &BlockID {
        match &self {
            Block::GenesisBlock(id) => id,
            Block::NetworkBlock(id, _) => id,
        }
    }

    pub fn parents(&self) -> &[BlockID] {
        match &self {
            Block::GenesisBlock(_) => &[],
            Block::NetworkBlock(_, network_block) => network_block.parents.as_slice(),
        }
    }
}

impl From<NetworkBlock> for Block {
    fn from(value: NetworkBlock) -> Self {
        Block::NetworkBlock(BlockID::new(&value), value)
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Block::GenesisBlock(id) => write!(f, "GenesisBlock({:?})", id),
            Block::NetworkBlock(id, _) => write!(f, "NetworkBlock({:?})", id),
        }
    }
}

mod network_block {}
