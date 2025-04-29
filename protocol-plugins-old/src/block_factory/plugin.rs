use std::sync::Arc;

use common::{
    blocks::{Block, NetworkBlock},
    ids::IssuerID,
    plugins::{Plugin, PluginRegistry},
};
use protocol::{ProtocolConfig, ProtocolPlugin};

use crate::tip_selection::TipSelection;

pub struct BlockFactory {
    tip_selection: Arc<TipSelection>,
}

impl BlockFactory {
    pub fn new_block(&self, issuer: &IssuerID) -> Block {
        Block::from(NetworkBlock {
            parents: self.tip_selection.get(),
            issuer_id: issuer.clone(),
        })
    }
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for BlockFactory {
    fn shutdown(&self) {}
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for BlockFactory {
    fn construct(dependencies: &mut PluginRegistry<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Arc::new(Self {
            tip_selection: dependencies.load(),
        })
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}
