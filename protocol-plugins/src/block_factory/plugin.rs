use std::sync::Arc;

use common::{
    blocks::{Block, NetworkBlock},
    ids::IssuerID,
    plugins::{Plugin, PluginManager},
};
use protocol::{Protocol, ProtocolConfig, ProtocolPlugin};
use virtual_voting::Vote;

use crate::tip_selection::TipSelection;

pub struct BlockFactory<C: ProtocolConfig> {
    tip_selection: Arc<TipSelection<C>>,
}

impl<C: ProtocolConfig> BlockFactory<C> {
    pub fn new_block(&self, issuer: &IssuerID) -> Block {
        Block::from(NetworkBlock {
            parents: self.tip_selection.get(),
            issuer_id: issuer.clone(),
        })
    }
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for BlockFactory<C> {
    fn process_vote(&self, _protocol: &Protocol<C>, _vote: &Vote<C>) -> protocol::Result<()> {
        Ok(())
    }
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for BlockFactory<C> {
    fn construct(dependencies: &mut PluginManager<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Arc::new(Self {
            tip_selection: dependencies.load(),
        })
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}
