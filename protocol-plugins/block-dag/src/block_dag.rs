use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use block_storage::{Address, BlockStorage};
use common::{
    blocks::BlockMetadata,
    down,
    extensions::ArcExt,
    rx::{Callbacks, Event, Subscription},
    up, with,
};
use protocol::{ManagedPlugin, Plugins};
use tracing::{Span, info, info_span, trace};

use crate::BlockDAGMetadata;

pub struct BlockDAG {
    pub block_available: Event<BlockMetadata>,
    block_storage_subscription: Mutex<Option<Subscription<Callbacks<Address>>>>,
    block_storage: Arc<BlockStorage>,
    span: Span,
}

#[async_trait]
impl ManagedPlugin for BlockDAG {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|this: &Weak<Self>| {
            let block_storage = plugins.load::<BlockStorage>();

            Self {
                block_available: Event::default(),
                block_storage_subscription: Mutex::new(Some(block_storage.new_address.subscribe(
                    with!(this: move |address| {
                        address.attach(with!(this: move |block| up!(this: {
                            this.provide_metadata(block)
                        })))
                    }),
                ))),
                block_storage,
                span: info_span!("block_dag"),
            }
        })
    }

    async fn shutdown(&self) {
        trace!("shutting down");
        self.block_storage_subscription.lock().unwrap().take();
        info!("stopped");
    }

    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl BlockDAG {
    fn provide_metadata(self: Arc<Self>, block: &BlockMetadata) {
        let metadata = block.set(Arc::new(BlockDAGMetadata::new(block.block.parents().len())));

        metadata.all_parents_available.attach({
            let this = self.downgrade();
            down!(block: move |_| up!(this, block: {
                this.block_available.trigger(&block)
            }))
        });

        for (index, parent_id) in block.block.parents().iter().enumerate() {
            self.block_storage.address(parent_id).attach(
                down!(metadata: move |parent| up!(metadata: {
                    metadata.register_parent(index, parent)
                })),
            );
        }
    }
}
