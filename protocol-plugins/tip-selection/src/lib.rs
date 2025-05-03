use std::{
    collections::HashSet,
    marker::PhantomData,
    sync::{Arc, Mutex, Weak},
};

use block_dag::{BlockDAG, BlockDAGMetadata};
use common::{
    blocks::BlockMetadata,
    errors::Result,
    ids::BlockID,
    rx::{Callbacks, Subscription},
};
use protocol::{ManagedPlugin, Plugins};
use virtual_voting::{VirtualVotingConfig, Vote};

#[derive(Default)]
pub struct TipSelection<C: VirtualVotingConfig> {
    tips: Mutex<HashSet<BlockMetadata>>,
    block_dag_subscription: Mutex<Option<Subscription<Callbacks<BlockMetadata>>>>,

    _marker: PhantomData<C>,
}

impl<C: VirtualVotingConfig> ManagedPlugin for TipSelection<C> {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|this: &Weak<Self>| {
            let block_dag = plugins.load::<BlockDAG>();

            Self {
                tips: Default::default(),
                block_dag_subscription: Mutex::new(Some(
                    block_dag.plugin_subscribe_block_and_metadata_available(
                        this,
                        |this, block, _: &Vote<C>| {
                            if let Err(err) = this.process_block(&block) {
                                // TODO: handle the error more elegantly
                                println!("{:?}", err);
                            }
                        },
                    ),
                )),
                _marker: PhantomData,
            }
        })
    }

    fn shutdown(&self) {
        self.block_dag_subscription.lock().unwrap().take();
    }
}

impl<C: VirtualVotingConfig> TipSelection<C> {
    pub fn get(&self) -> Vec<BlockID> {
        self.tips
            .lock()
            .expect("failed to lock")
            .iter()
            .map(|x| x.block.id())
            .cloned()
            .collect()
    }

    fn process_block(&self, block: &BlockMetadata) -> Result<()> {
        let metadata = block;

        let block_dag_metadata = metadata.try_get::<Arc<BlockDAGMetadata>>()?;
        let locked_parents = block_dag_metadata.parents();
        let parent_refs = locked_parents.iter();
        let mut removed_tips = Vec::with_capacity(block.block.parents().len());
        let mut tips = self.tips.lock().expect("failed to lock");
        for parent_ref in parent_refs {
            match parent_ref.try_upgrade() {
                Ok(parent) => {
                    if tips.remove(&parent) {
                        removed_tips.push(parent);
                    }
                }
                Err(err) => {
                    tips.extend(removed_tips.into_iter());

                    return Err(err);
                }
            }
        }
        tips.insert(metadata.clone());

        Ok(())
    }
}
