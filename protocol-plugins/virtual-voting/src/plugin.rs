use std::{
    marker::PhantomData,
    sync::{Arc, Mutex, Weak},
};

use block_dag::{BlockDAG, BlockDAGMetadata};
use common::{
    blocks::{Block, BlockMetadata, BlockMetadataRef},
    errors::Error::BlockNotFound,
    rx::{Callbacks, Subscription},
};
use protocol::{ManagedPlugin, Plugins};

use crate::{Error, Result, VirtualVotingConfig, Vote, Votes};

pub struct VirtualVoting<C: VirtualVotingConfig<Source = BlockMetadataRef>> {
    subscription: Mutex<Option<Subscription<Callbacks<BlockMetadata>>>>,
    _marker: PhantomData<C>,
}

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> VirtualVoting<C> {
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
                None => return Err(Error::CommonError(BlockNotFound)),
            };
        }

        Ok(result)
    }
}

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> ManagedPlugin for VirtualVoting<C> {
    fn construct(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|_virtual_voting: &Weak<Self>| {
            let block_dag: Arc<BlockDAG> = plugins.load();
            let config: Arc<C> = plugins.get().unwrap();

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
                                            block.metadata().set(Arc::new(vote));
                                        }
                                        Err(_err) => {}
                                    },
                                    Err(_err) => {}
                                }
                            }
                            _ => block.metadata().set(Arc::new(Vote::new_genesis(
                                block.downgrade(),
                                config.clone(),
                            ))),
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
