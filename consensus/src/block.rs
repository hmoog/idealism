use types::BlockID;
use utils::Id;
use virtual_voting::Config;

pub enum Block<C: Config> {
    GenesisBlock(genesis_block::Details<C>),
    NetworkBlock(network_block::Details<C>),
}

impl<C: Config> Block<C> {
    pub fn issuer_id(&self) -> &Id<C::IssuerID> {
        match &self {
            Block::GenesisBlock(genesis_block) => &genesis_block.issuer_id,
            Block::NetworkBlock(network_block) => &network_block.issuer_id,
        }
    }
}

pub mod genesis_block {
    use types::BlockID;
    use virtual_voting::Config;
    use crate::issuer_id::IssuerID;

    pub struct Details<C: Config> {
        pub id: BlockID,
        pub issuer_id: IssuerID<C>,
    }
}

pub mod network_block {
    use types::BlockID;
    use virtual_voting::Config;
    use crate::issuer_id::IssuerID;

    pub struct Details<C: Config> {
        pub id: BlockID,
        pub parents: Vec<BlockID>,
        pub issuer_id: IssuerID<C>,
    }
}

impl<C: Config> blockdag::Block for Block<C> {
    type ID = BlockID;

    fn id(&self) -> &Self::ID {
        match &self {
            Block::GenesisBlock(genesis_block) => &genesis_block.id,
            Block::NetworkBlock(network_block) => &network_block.id,
        }
    }

    fn parents(&self) -> &[Self::ID] {
        match &self {
            Block::GenesisBlock(_) => &[],
            Block::NetworkBlock(network_block) => network_block.parents.as_slice(),
        }
    }
}
