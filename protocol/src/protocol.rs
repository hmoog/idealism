use std::sync::Arc;

use blockdag::{BlockDAG, BlockMetadata};
use common::{blocks::Block, plugins::PluginManager, rx::ResourceGuard};
use virtual_voting::Vote;
use zero::{Clone0, Deref0};

use crate::{ProtocolConfig, ProtocolPlugin, Result};

#[derive(Deref0, Clone0, Default)]
pub struct Protocol<C: ProtocolConfig>(Arc<ProtocolData<C>>);

#[derive(Default)]
pub struct ProtocolData<C: ProtocolConfig> {
    pub config: Arc<C>,
    pub block_dag: BlockDAG<C>,
    pub plugins: PluginManager<dyn ProtocolPlugin<C>>,
}

impl<C: ProtocolConfig> Protocol<C> {
    pub fn new(config: C) -> Self {
        Self(Arc::new(ProtocolData {
            config: Arc::new(config),
            block_dag: BlockDAG::default(),
            plugins: PluginManager::default(),
        }))
    }

    pub fn init(&self) {
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

        let genesis_block = Block::GenesisBlock(self.config.genesis_block_id());
        self.block_dag.attach(genesis_block);
    }

    fn process_block(&self, metadata: &ResourceGuard<BlockMetadata<C>>) -> Result<()> {
        metadata.vote.set(match &metadata.block {
            Block::NetworkBlock(_, network_block) => Vote::new(
                metadata.downgrade(),
                &network_block.issuer_id,
                0,
                metadata.referenced_votes()?,
            )?,
            _ => Vote::new_genesis(metadata.downgrade(), self.config.clone()),
        });

        self.plugins
            .for_each(|plugin| plugin.process_block(metadata))
    }
}
