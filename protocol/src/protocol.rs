use std::sync::Arc;

use blockdag::{BlockDAG, BlockMetadata};
use types::{
    blocks::{Block, NetworkBlock},
    ids::IssuerID,
    rx::ResourceGuard,
};
use virtual_voting::Vote;
use zero::{Clone0, Deref0};

use crate::{
    Config, Events, State,
    error::{Error, Result},
    tips::Tips,
};

#[derive(Deref0, Clone0, Default)]
pub struct Protocol<C: Config>(Arc<ProtocolData<C>>);

#[derive(Default)]
pub struct ProtocolData<C: Config> {
    pub events: Events<C>,
    pub block_dag: BlockDAG<C>,
    pub state: State<C>,
    pub tips: Tips<C>,
}

impl<C: Config> Protocol<C> {
    pub fn init(self, config: C) -> Self {
        let genesis_block = Block::GenesisBlock(config.genesis_block_id());
        self.block_dag.init(genesis_block, config);

        let genesis_metadata = self.block_dag.genesis();
        let genesis_vote = genesis_metadata.vote().expect("must exist");
        self.state.init(genesis_vote);
        self.tips.init(genesis_metadata);

        self.block_dag
            .on_block_ready({
                let protocol = self.clone();

                move |block_metadata| {
                    if let Err(err) = protocol.process_block(block_metadata) {
                        protocol.events.error.trigger(&err);
                    }
                }
            })
            .forever();

        self
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

                self.state.apply(vote.clone())?;
                self.tips.apply(metadata)?;

                metadata.vote.set(vote);

                Ok(())
            }
            _ => Err(Error::UnsupportedBlockType),
        }
    }
}
