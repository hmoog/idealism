use std::sync::Arc;

use blockdag::{BlockDAG, BlockMetadata};
use common::{blocks::Block, plugins::PluginRegistry, rx::ResourceGuard};
use virtual_voting::Vote;
use zero::{Clone0, Deref0};

use crate::{ProtocolConfig, ProtocolPlugin, ProtocolResult};

#[derive(Deref0, Clone0)]
pub struct Protocol<C: ProtocolConfig>(Arc<ProtocolData<C>>);

pub struct ProtocolData<C: ProtocolConfig> {
    pub block_dag: BlockDAG<C>,
    pub plugins: PluginRegistry<dyn ProtocolPlugin<C>>,
    pub config: Arc<C>,
}

impl<C: ProtocolConfig> Protocol<C> {
    pub fn new(config: C) -> Self {
        let protocol = Self(Arc::new(ProtocolData {
            block_dag: BlockDAG::default(),
            plugins: config.inject_plugins(PluginRegistry::default()),
            config: Arc::new(config),
        }));

        let subscription = protocol.block_dag.subscribe({
            let protocol = protocol.clone();
            move |block| {
                if let Err(err) = protocol.process_block(block) {
                    block.error.set(err);
                }
            }
        });
        subscription.retain();

        let genesis_block = Block::GenesisBlock(protocol.config.genesis_block_id());
        protocol.block_dag.queue(genesis_block);

        protocol
    }

    fn process_block(&self, metadata: &ResourceGuard<BlockMetadata<C>>) -> ProtocolResult<()> {
        metadata.signal().set(match &metadata.block {
            Block::NetworkBlock(_, network_block) => Vote::new(
                metadata.downgrade(),
                &network_block.issuer_id,
                0,
                metadata.referenced_votes()?,
            )?,
            _ => Vote::new_genesis(metadata.downgrade(), self.config.clone()),
        });

        for plugin in self.plugins.iter() {
            plugin.process_block(metadata)?;
        }

        Ok(())
    }
}
