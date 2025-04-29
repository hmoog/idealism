use std::sync::Arc;

use common::{
    blocks::{Block, NetworkBlock},
    ids::IssuerID,
    plugins::{Plugin, PluginRegistry},
};
use protocol::{ProtocolConfig, ProtocolPlugin};

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

impl<C: ProtocolConfig> ProtocolPlugin for BlockFactory<C> {
    fn shutdown(&self) {}
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin> for BlockFactory<C> {
    fn construct(dependencies: &mut PluginRegistry<dyn ProtocolPlugin>) -> Arc<Self> {
        Arc::new(Self {
            tip_selection: dependencies.load(),
        })
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin> {
        arc
    }
}
