use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, Weak},
};

use block_dag::{BlockDAG, BlockDAGMetadata};
use common::{
    blocks::{Block, BlockMetadata, BlockMetadataRef},
    rx::{Callbacks, Subscription},
};
use protocol::{ManagedPlugin, Plugins};

use crate::{Result, VirtualVotingConfig, Vote, Votes};

pub struct VirtualVoting<C: VirtualVotingConfig> {
    subscription: Mutex<Option<Subscription<Callbacks<BlockMetadata>>>>,
    _marker: PhantomData<C>,
}

impl<C: VirtualVotingConfig> VirtualVoting<C> {
    pub fn referenced_votes(block: &BlockMetadata) -> Result<Votes<C>> {
        let mut result = Votes::default();
        for block_ref in block
            .try_get::<Arc<BlockDAGMetadata>>()?
            .parents
            .read()
            .unwrap()
            .iter()
        {
            result.insert(block_ref.try_upgrade()?.try_get::<Vote<C>>()?);
        }

        Ok(result)
    }
}

impl<C: VirtualVotingConfig> ManagedPlugin for VirtualVoting<C> {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|_virtual_voting: &Weak<Self>| {
            let block_dag: Arc<BlockDAG> = plugins.load();
            let config: Arc<C> = plugins.get().unwrap();

            Self {
                subscription: Mutex::new(Some(block_dag.block_available.subscribe({
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
                                        Err(err) => {
                                            println!("Error creating vote1: {:?}", err);
                                        }
                                    },
                                    Err(err) => {
                                        println!("Error creating vote2: {:?}", err);
                                    }
                                }
                            }
                            _ => {
                                block
                                    .metadata()
                                    .set(Vote::new_genesis(block.downgrade(), config.clone()));
                            }
                        };
                    }
                }))),
                _marker: PhantomData,
            }
        })
    }

    fn shutdown(&self) {
        self.subscription.lock().unwrap().take();
    }
}
