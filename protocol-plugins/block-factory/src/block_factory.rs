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

impl<C: VirtualVotingConfig> ManagedPlugin for BlockFactory<C> {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new(Self {
            tip_selection: plugins.load(),
        })
    }
}

impl<C: VirtualVotingConfig> BlockFactory<C> {
    pub fn create_block(&self, issuer: &IssuerID) -> Block {
        Block::from(NetworkBlock {
            parents: self.tip_selection.get(),
            issuer_id: issuer.clone(),
        })
    }
}
