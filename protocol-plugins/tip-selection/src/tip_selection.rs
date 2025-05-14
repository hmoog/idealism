use std::{
    collections::HashSet,
    marker::PhantomData,
    sync::{Arc, Mutex, Weak},
};

use block_dag::{BlockDAG, BlockDAGMetadata};
use common::{
    blocks::BlockMetadata,
    down,
    errors::Result,
    ids::BlockID,
    rx::{Callbacks, Subscription},
    up, with,
};
use protocol::{ManagedPlugin, Plugins};
use tracing::{Span, info_span};
use virtual_voting::{VirtualVotingConfig, Vote};

use crate::TipSelectionMetadata;

pub struct TipSelection<C: VirtualVotingConfig> {
    tips: Mutex<HashSet<BlockMetadata>>,
    block_dag_subscription: Mutex<Option<Subscription<Callbacks<BlockMetadata>>>>,
    span: Span,
    _marker: PhantomData<C>,
}

impl<C: VirtualVotingConfig> ManagedPlugin for TipSelection<C> {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|this: &Weak<Self>| {
            let block_dag = plugins.load::<BlockDAG>();

            Self {
                tips: Default::default(),
                block_dag_subscription: Mutex::new(Some(block_dag.block_available.subscribe(
                    with!(this: move |block| with!(this: {
                        block.attach::<Vote<C>>(down!(block: move |_| up!(this, block: {
                            this.process_block(&block).unwrap_or_else(|e| println!("{:?}", e))
                        })))
                    })),
                ))),
                span: info_span!("tip_selection"),
                _marker: PhantomData,
            }
        })
    }

    fn shutdown(&self) {
        self.block_dag_subscription.lock().unwrap().take();
    }

    fn span(&self) -> Span {
        self.span.clone()
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

        block.metadata().set(Arc::new(TipSelectionMetadata));

        Ok(())
    }
}
