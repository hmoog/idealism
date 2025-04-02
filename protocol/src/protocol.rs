use std::{
    any::Any,
    sync::{Arc, RwLock},
};

use blockdag::{BlockDAG, BlockMetadata};
use common::{
    blocks::{Block, NetworkBlock},
    ids::IssuerID,
    plugins::{Plugin, PluginManager},
    rx::ResourceGuard,
};
use virtual_voting::Vote;
use zero::{Clone0, Deref0};

use crate::{Error, ProtocolConfig, ProtocolPlugin, Result, Tips};

#[derive(Deref0, Clone0, Default)]
pub struct Protocol<C: ProtocolConfig>(Arc<ProtocolData<C>>);

#[derive(Default)]
pub struct ProtocolData<C: ProtocolConfig> {
    pub block_dag: BlockDAG<C>,
    pub tips: Tips<C>,
    pub plugins: RwLock<PluginManager<dyn ProtocolPlugin<C>>>,
}

impl<C: ProtocolConfig> Protocol<C> {
    pub fn init(self, config: C) -> Self {
        self.block_dag
            .on_block_ready({
                let protocol = self.clone();
                move |block_metadata| {
                    if let Err(err) = protocol.process_block(block_metadata) {
                        block_metadata.error.set(err);
                    }
                }
            })
            .forever();

        self.block_dag
            .init(Block::GenesisBlock(config.genesis_block_id()), config);

        self.tips.init(&self);

        self
    }

    pub fn load_plugin<U: Any + Send + Sync + Plugin<dyn ProtocolPlugin<C>> + 'static>(
        &self,
    ) -> Arc<U> {
        let mut plugins = self.plugins.write().unwrap();
        plugins.load::<U>()
    }

    pub fn new_block(&self, issuer: &IssuerID) -> Block {
        Block::from(NetworkBlock {
            parents: self.tips.get(),
            issuer_id: issuer.clone(),
        })
    }

    fn process_block(&self, metadata: &ResourceGuard<BlockMetadata<C>>) -> Result<()> {
        match &metadata.block {
            Block::NetworkBlock(_id, network_block) => {
                let vote = Vote::new(
                    metadata.downgrade(),
                    &network_block.issuer_id,
                    0,
                    metadata.referenced_votes()?,
                )?;

                self.tips.process_vote(metadata)?;

                for plugin in self.plugins.read().unwrap().iter() {
                    plugin.process_vote(self, &vote)?;
                }

                metadata.vote.set(vote);

                Ok(())
            }
            _ => Err(Error::UnsupportedBlockType),
        }
    }
}
