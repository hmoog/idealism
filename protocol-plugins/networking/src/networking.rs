use std::sync::Arc;

use async_trait::async_trait;
use common::{
    blocks::Block,
    networking::{Endpoint, Network},
    traced,
};
use inbox::Inbox;
use outbox::Outbox;
use protocol::{ManagedPlugin, Plugins};
use tokio::{
    sync::{
        Mutex, MutexGuard,
        mpsc::{UnboundedReceiver, UnboundedSender},
        watch,
        watch::Receiver,
    },
    task::JoinHandle,
};
use tracing::{Level, Span, error, info_span, span, trace};

pub struct Networking {
    inbox: Arc<Inbox>,
    outbox: Arc<Outbox>,
    workers: Mutex<Option<(JoinHandle<()>, JoinHandle<()>, watch::Sender<()>)>>,
    span: Span,
}

#[async_trait]
impl ManagedPlugin for Networking {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new(Self {
            inbox: plugins.load(),
            outbox: plugins.load(),
            workers: Mutex::new(None),
            span: info_span!("networking"),
        })
    }

    async fn shutdown(&self) {
        self.disconnect().await;
    }

    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Networking {
    pub async fn connect<N: Network>(&self, network: &N) {
        let Endpoint { inbound, outbound } = network.endpoint().await;
        let mut workers = self.workers.lock().await;
        self.shutdown_workers(&mut workers).await;

        let (shutdown_signal, is_shutdown) = watch::channel(());
        *workers = Some((
            self.inbound_worker(inbound, is_shutdown.clone()),
            self.outbound_worker(outbound, is_shutdown.clone()),
            shutdown_signal,
        ));
    }

    pub async fn disconnect(&self) {
        self.shutdown_workers(&mut self.workers.lock().await).await;
    }

    async fn shutdown_workers(
        &self,
        workers: &mut MutexGuard<'_, Option<(JoinHandle<()>, JoinHandle<()>, watch::Sender<()>)>>,
    ) {
        if let Some((inbound_worker, outbound_worker, shutdown)) = workers.take() {
            drop(shutdown); // close the shutdown channel to signal workers to stop
            // wait for workers to finish
            let _ = inbound_worker.await;
            let _ = outbound_worker.await;
        }
    }

    fn inbound_worker(
        &self,
        mut receiver: UnboundedReceiver<Block>,
        mut is_shutdown: Receiver<()>,
    ) -> JoinHandle<()> {
        let inbox = self.inbox.clone();
        traced::worker(
            async move {
                loop {
                    tokio::select! {
                        Some(block) = receiver.recv() => {
                            let id = block.id().clone();
                            if let Err(e) = inbox.send(block) {
                                error!("failed to receive block (id={:?}): {:?}", id, e);
                            } else {
                                trace!("received block (id={:?})", id);
                            }
                        },
                        _ = is_shutdown.changed() => break, // channel closed = shutdown
                    }
                }
            },
            span!(parent: self.span.clone(), Level::INFO, "inbound"),
        )
    }

    fn outbound_worker(
        &self,
        sender: UnboundedSender<Block>,
        mut is_shutdown: Receiver<()>,
    ) -> JoinHandle<()> {
        let outbox = self.outbox.clone();
        traced::worker(
            async move {
                let mut outbox = outbox.receiver.lock().await;
                loop {
                    tokio::select! {
                        Some(block) = outbox.recv() => {
                            let id = block.id().clone();
                            if let Err(e) = sender.send(block) {
                                error!("failed to send block (id={:?}): {:?}", id, e);
                            } else {
                                trace!("sent block (id={:?})", id);
                            }
                        },
                        _ = is_shutdown.changed() => break, // channel closed = shutdown
                    }
                }
            },
            span!(parent: self.span.clone(), Level::INFO, "outbound"),
        )
    }
}
