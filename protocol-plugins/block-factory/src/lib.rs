use std::sync::Arc;

use common::{
    blocks::{Block, BlockMetadataRef, NetworkBlock},
    ids::IssuerID,
    plugins::{Plugin, PluginRegistry},
};
use protocol::ProtocolPlugin;
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

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> ProtocolPlugin for BlockFactory<C> {
    fn shutdown(&self) {}
}

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> Plugin<dyn ProtocolPlugin>
    for BlockFactory<C>
{
    fn construct(dependencies: &mut PluginRegistry<dyn ProtocolPlugin>) -> Arc<Self> {
        Arc::new(Self {
            tip_selection: dependencies.load(),
        })
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin> {
        arc
    }
}
