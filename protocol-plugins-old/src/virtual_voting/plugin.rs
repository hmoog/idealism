use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, Weak},
};

use block_dag::{BlockDAG, BlockDAGMetadata};
use common::{
    blocks::{Block, BlockMetadata, BlockMetadataRef},
    errors::{Error::BlockNotFound, Result},
    plugins::{Plugin, PluginRegistry},
    rx::{Callbacks, Subscription},
};
use protocol::{ProtocolConfig, ProtocolPlugin};
use virtual_voting::{Vote, Votes};

pub struct VirtualVoting<C: ProtocolConfig> {
    subscription: Mutex<Option<Subscription<Callbacks<BlockMetadata>>>>,
    _marker: PhantomData<C>,
}

impl<C: ProtocolConfig> VirtualVoting<C> {
    pub fn referenced_votes(block: &BlockMetadata) -> Result<Votes<C>> {
        let mut result = Votes::default();
        for block_ref in block
            .try_get::<Arc<BlockDAGMetadata>>()?
            .parents
            .lock()
            .unwrap()
            .iter()
        {
            match block_ref.upgrade() {
                Some(block) => result.insert(block.try_get::<Vote<C>>()?),
                None => return Err(BlockNotFound),
            };
        }

        Ok(result)
    }
}

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for VirtualVoting<C> {
    fn construct(plugins: &mut PluginRegistry<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Arc::new_cyclic(|_virtual_voting: &Weak<Self>| {
            let block_dag: Arc<BlockDAG> = plugins.load();

            Self {
                subscription: Mutex::new(Some(block_dag.subscribe({
                    move |block| {
                        match &block.block {
                            Block::NetworkBlock(_, network_block) => {
                                let src: BlockMetadataRef = block.downgrade();
                                match Self::referenced_votes(block) {
                                    Ok(referenced_votes) => match Vote::new(
                                        src,
                                        &network_block.issuer_id,
                                        0,
                                        referenced_votes,
                                    ) {
                                        Ok(vote) => {
                                            block.metadata().set(vote);
                                        }
                                        Err(_) => {}
                                    },
                                    Err(_) => {}
                                }
                            }
                            _ => {
                                // Vote::new_genesis(block.downgrade(), self.config.clone())
                            }
                        };
                    }
                }))),
                _marker: PhantomData,
            }
        })
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for VirtualVoting<C> {
    fn shutdown(&self) {
        self.subscription.lock().unwrap().take();
    }
}
