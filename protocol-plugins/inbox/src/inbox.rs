use std::{
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
};

use block_storage::BlockStorage;
use common::{blocks::Block, down, extensions::ArcExt, up, with};
use protocol::{ManagedPlugin, Plugins};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task,
};
use tracing::{Level, Span, debug, error, span, trace};

pub struct Inbox {
    sender: RwLock<Option<UnboundedSender<Block>>>,
    receiver: Arc<Mutex<UnboundedReceiver<Block>>>,
    num_workers: usize,
    block_storage: Arc<BlockStorage>,
}

impl ManagedPlugin for Inbox {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        let (tx, rx) = unbounded_channel();
        Arc::new(Self {
            sender: RwLock::new(Some(tx)),
            receiver: Arc::new(Mutex::new(rx)),
            num_workers: 2,
            block_storage: plugins.load(),
        })
    }

    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        let num_workers = self.num_workers;
        let block_storage = self.block_storage.clone();
        let rx = Arc::clone(&self.receiver);

        Some(Box::pin(async move {
            let mut worker_handles = Vec::new();
            for i in 0..num_workers {
                let worker_span = span!(parent: Span::current(), Level::INFO, "worker", id = i);
                worker_handles.push((
                    task::spawn_blocking(with!(worker_span: down!(block_storage, rx: move || {
                        up!(block_storage, rx: worker_span.in_scope(|| {
                            debug!(target: "inbox", "started");
                            while let Some(block) = rx.lock().unwrap().blocking_recv() {
                                span!(parent: worker_span.clone(), Level::INFO, "block", block_id = i).in_scope(|| {
                                    debug!(target: "inbox", "received");
                                    block_storage.insert(block);
                                })
                            }
                            debug!(target: "inbox", "stopped");
                        }))
                    })),
                    ),
                    worker_span,
                ));
            }

            for (worker_handle, worker_span) in worker_handles.into_iter() {
                if let Err(e) = worker_handle.await {
                    worker_span.in_scope(|| error!(target: "inbox", "panicked: {e}"))
                }
            }
        }))
    }

    fn shutdown(&self) {
        trace!(target: "inbox", "closing inbox");
        self.sender.write().unwrap().take();
    }
}

impl Inbox {
    pub fn send(&self, block: Block) -> Result<(), tokio::sync::mpsc::error::SendError<Block>> {
        if let Some(sender) = &*self.sender.read().unwrap() {
            sender.send(block)
        } else {
            Err(tokio::sync::mpsc::error::SendError(block))
        }
    }
}
