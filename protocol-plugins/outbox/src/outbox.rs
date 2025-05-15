use std::sync::{Arc, Mutex};

use block_dag::BlockDAG;
use common::{
    blocks::{Block, BlockMetadata},
    down,
    rx::{Callbacks, Subscription},
    up, with,
};
use protocol::{ManagedPlugin, Plugins};
use tip_selection::TipSelectionMetadata;
use tokio::sync::{
    Mutex as AsyncMutex,
    mpsc::{UnboundedReceiver, unbounded_channel},
};
use tracing::{Span, error, info_span, trace};

pub struct Outbox {
    pub receiver: AsyncMutex<UnboundedReceiver<Block>>,
    block_dag_subscription: Mutex<Option<Subscription<Callbacks<BlockMetadata>>>>,
    span: Span,
}

impl ManagedPlugin for Outbox {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        let block_dag = plugins.load::<BlockDAG>();
        let (tx, rx) = unbounded_channel();

        Arc::new(Self {
            receiver: AsyncMutex::new(rx),
            block_dag_subscription: Mutex::new(Some(block_dag.block_available.subscribe(
                move |block| {
                    block.attach::<Arc<TipSelectionMetadata>>(with!(tx: down!(block: {
                        move |_| up!(block: {
                            if let Err(e) = tx.send(block.block.clone()) {
                                error!("failed to send block: {:?}", e);
                            } else {
                                trace!("forwarded block");
                            }
                        })
                    })))
                },
            ))),
            span: info_span!("outbox"),
        })
    }

    fn shutdown(&self) {
        trace!("unsubscribing from BlockDAG");
        self.block_dag_subscription.lock().unwrap().take();
    }

    fn span(&self) -> Span {
        self.span.clone()
    }
}
