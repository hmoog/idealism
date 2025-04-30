use std::sync::Arc;

use common::{
    blocks::{Block, NetworkBlock},
    ids::IssuerID,
};
use protocol::{ManagedPlugin, Plugins};
use tip_selection::TipSelection;
use virtual_voting::VirtualVotingConfig;

pub struct BlockFactory<C: VirtualVotingConfig> {
    tip_selection: Arc<TipSelection<C>>,
}

impl<C: VirtualVotingConfig> BlockFactory<C> {
    pub fn new_block(&self, issuer: &IssuerID) -> Block {
        Block::from(NetworkBlock {
            parents: self.tip_selection.get(),
            issuer_id: issuer.clone(),
        })
    }
}

impl<C: VirtualVotingConfig> ManagedPlugin for BlockFactory<C> {
    fn construct(dependencies: &mut Plugins) -> Arc<Self> {
        Arc::new(Self {
            tip_selection: dependencies.load(),
        })
    }
}
