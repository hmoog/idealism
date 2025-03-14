use std::{fmt, fmt::Debug};

pub use network_block::*;
use types::BlockID;
use virtual_voting::Config;

pub enum Block<C: Config> {
    GenesisBlock(BlockID),
    NetworkBlock(BlockID, NetworkBlock<C>),
}

impl<C: Config> From<NetworkBlock<C>> for Block<C> {
    fn from(value: NetworkBlock<C>) -> Self {
        Block::NetworkBlock(BlockID::new(&value), value)
    }
}

impl<C: Config> blockdag::Block for Block<C> {
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

impl<C: Config> Debug for Block<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Block::GenesisBlock(id) => write!(f, "GenesisBlock({:?})", id),
            Block::NetworkBlock(id, _) => write!(f, "NetworkBlock({:?})", id),
        }
    }
}

mod network_block {
    use types::{BlockID, Hashable, Hasher};
    use virtual_voting::Config;

    use crate::issuer_id::IssuerID;

    pub struct NetworkBlock<C: Config> {
        pub parents: Vec<BlockID>,
        pub issuer_id: IssuerID<C>,
    }

    impl<C: Config> Hashable for NetworkBlock<C> {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            hasher.update(&self.parents.len().to_be_bytes());
            for parent in &self.parents {
                hasher.update(parent.as_slice());
            }
        }
    }
}
