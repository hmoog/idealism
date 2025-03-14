use std::{fmt, fmt::Debug};

pub use network_block::*;
use types::BlockID;

pub enum Block {
    GenesisBlock(BlockID),
    NetworkBlock(BlockID, NetworkBlock),
}

impl From<NetworkBlock> for Block {
    fn from(value: NetworkBlock) -> Self {
        Block::NetworkBlock(BlockID::new(&value), value)
    }
}

impl blockdag::Block for Block {
    type ID = BlockID;

    fn id(&self) -> &Self::ID {
        match &self {
            Block::GenesisBlock(id) => id,
            Block::NetworkBlock(id, _) => id,
        }
    }

    fn parents(&self) -> &[Self::ID] {
        match &self {
            Block::GenesisBlock(_) => &[],
            Block::NetworkBlock(_, network_block) => network_block.parents.as_slice(),
        }
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
    use types::{BlockID, Hashable, Hasher};

    use crate::issuer_id::IssuerID;

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
