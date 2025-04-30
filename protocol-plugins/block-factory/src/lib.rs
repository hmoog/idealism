use std::sync::Arc;

use common::{
    blocks::{Block, BlockMetadataRef, NetworkBlock},
    ids::IssuerID,
    plugins::{ManagedPlugin, Plugins},
};
use tip_selection::TipSelection;
use virtual_voting::VirtualVotingConfig;

pub struct BlockFactory<C: VirtualVotingConfig<Source = BlockMetadataRef>> {
    tip_selection: Arc<TipSelection<C>>,
}

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> BlockFactory<C> {
    pub fn new_block(&self, issuer: &IssuerID) -> Block {
        Block::from(NetworkBlock {
            parents: self.tip_selection.get(),
            issuer_id: issuer.clone(),
        })
    }
}

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> ManagedPlugin for BlockFactory<C> {
    fn construct(dependencies: &mut Plugins) -> Arc<Self> {
        Arc::new(Self {
            tip_selection: dependencies.load(),
        })
    }

    fn shutdown(&self) {}
}
