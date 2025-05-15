use std::sync::{Arc, Mutex, RwLock};

use async_trait::async_trait;
use block_storage::BlockStorage;
use common::{blocks::Block, down, extensions::ArcExt, up, with};
use protocol::{ManagedPlugin, Plugins};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task,
    task::JoinHandle,
};
use tracing::{Instrument, Span, debug, error, info, info_span, trace};

pub struct Inbox {
    sender: RwLock<Option<UnboundedSender<Block>>>,
    receiver: Arc<Mutex<UnboundedReceiver<Block>>>,
    num_workers: usize,
    block_storage: Arc<BlockStorage>,
    span: Span,
    worker_handles: tokio::sync::Mutex<Option<Vec<JoinHandle<()>>>>,
}

#[async_trait]
impl ManagedPlugin for Inbox {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        let (tx, rx) = unbounded_channel();
        Arc::new(Self {
            sender: RwLock::new(Some(tx)),
            receiver: Arc::new(Mutex::new(rx)),
            num_workers: 2,
            block_storage: plugins.load(),
            span: info_span!("inbox"),
            worker_handles: tokio::sync::Mutex::new(None),
        })
    }

    async fn start(&self) {
        let num_workers = self.num_workers;
        let block_storage = self.block_storage.clone();
        let rx = Arc::clone(&self.receiver);

        let mut worker_handles = Vec::new();
        for i in 0..num_workers {
            let handle = tokio::spawn(with!(block_storage, rx: async move {
                let worker_span = Span::current();
                let worker_task = task::spawn_blocking(with!(worker_span: down!(block_storage, rx: move || {
                    up!(block_storage, rx: worker_span.in_scope(|| {
                        debug!("worker started");
                        while let Some(block) = rx.lock().unwrap().blocking_recv() {
                            info_span!("block", id = %block.id()).in_scope(|| {
                                debug!("block received");
                                block_storage.insert(block);
                            })
                        }
                        debug!("worker stopped");
                    }))
                })));

                if let Err(e) = worker_task.await {
                    error!("worker panicked: {e}");
                }
            }).instrument(info_span!("worker", id = i)));

            worker_handles.push(handle);
        }

        *self.worker_handles.lock().await = Some(worker_handles);
    }

    async fn shutdown(&self) {
        trace!("shutting down");
        self.sender.write().unwrap().take();

        if let Some(worker_handles) = self.worker_handles.lock().await.take() {
            for worker in worker_handles {
                let _ = worker.await;
            }
        }
        info!("stopped");
    }

    fn span(&self) -> Span {
        self.span.clone()
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
