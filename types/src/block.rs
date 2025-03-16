use std::{fmt, fmt::Debug};

pub use network_block::*;

use crate::ids::BlockID;

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

mod network_block {
    use crate::{
        hash::{Hashable, Hasher},
        ids::{BlockID, IssuerID},
    };

    pub struct NetworkBlock {
        pub parents: Vec<BlockID>,
        pub issuer_id: IssuerID,
    }

    impl Hashable for NetworkBlock {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            hasher.update(&self.parents.len().to_be_bytes());
            for parent in &self.parents {
                hasher.update(parent.as_slice());
            }
        }
    }
}
