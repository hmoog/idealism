use std::sync::Arc;

use common::{
    blocks::Block,
    networking::{Endpoint, Network},
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
use tracing::{Instrument, Level, Span, error, info, info_span, span, trace};

pub struct Networking {
    inbox: Arc<Inbox>,
    outbox: Arc<Outbox>,
    workers: Mutex<Option<(JoinHandle<()>, JoinHandle<()>, watch::Sender<()>)>>,
    span: Span,
}

impl ManagedPlugin for Networking {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new(Self {
            inbox: plugins.load(),
            outbox: plugins.load(),
            workers: Mutex::new(None),
            span: info_span!("networking"),
        })
    }

    fn shutdown(&self) {
        //
    }

    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Networking {
    pub async fn connect<N: Network>(&self, network: &N) {
        let Endpoint { inbound, outbound } = network.endpoint().await;

        let mut workers = self.workers.lock().await;
        let _ = self.shutdown_workers(&mut workers).await;

        // create new workers
        let (shutdown_signal, is_shutdown) = watch::channel(());
        *workers = Some((
            self.new_inbound_worker(self.inbox.clone(), inbound, is_shutdown.clone()),
            self.new_outbound_worker(self.outbox.clone(), outbound, is_shutdown.clone()),
            shutdown_signal,
        ));
    }

    pub async fn disconnect(&self) {
        let _ = self.shutdown_workers(&mut self.workers.lock().await).await;
    }

    async fn shutdown_workers(
        &self,
        workers: &mut MutexGuard<'_, Option<(JoinHandle<()>, JoinHandle<()>, watch::Sender<()>)>>,
    ) {
        if let Some((inbound_worker, outbound_worker, shutdown)) = workers.take() {
            drop(shutdown); // close the shutdown channel to signal workers to stop

            // wait for workers to finish
            if let Err(e) = inbound_worker.await {
                span!(parent: self.span.clone(), Level::INFO, "inbound")
                    .in_scope(|| error!("worker panicked: {:?}", e));
            }
            if let Err(e) = outbound_worker.await {
                span!(parent: self.span.clone(), Level::INFO, "outbound")
                    .in_scope(|| error!("worker panicked: {:?}", e));
            }
        }
    }

    fn new_inbound_worker(
        &self,
        inbox: Arc<Inbox>,
        mut receiver: UnboundedReceiver<Block>,
        mut is_shutdown: Receiver<()>,
    ) -> JoinHandle<()> {
        tokio::spawn(
            async move {
                info!("worker started");
                loop {
                    tokio::select! {
                        Some(block) = receiver.recv() => {
                            if let Err(e) = inbox.send(block) {
                                error!("failed to receive block: {:?}", e);
                            } else {
                                trace!("received block");
                            }
                        },
                        _ = is_shutdown.changed() => break, // channel closed = shutdown
                    }
                }
                info!("worker stopped");
            }
            .instrument(span!(parent: self.span.clone(), Level::INFO, "inbound")),
        )
    }

    fn new_outbound_worker(
        &self,
        outbox: Arc<Outbox>,
        sender: UnboundedSender<Block>,
        mut is_shutdown: Receiver<()>,
    ) -> JoinHandle<()> {
        tokio::spawn(
            async move {
                info!("worker started");
                let mut outbox = outbox.receiver.lock().await;
                loop {
                    tokio::select! {
                        Some(block) = outbox.recv() => {
                            if let Err(e) = sender.send(block) {
                                error!("failed to send block: {:?}", e);
                            } else {
                                trace!("sent block");
                            }
                        },
                        _ = is_shutdown.changed() => break, // channel closed = shutdown
                    }
                }
                info!("worker stopped");
            }
            .instrument(span!(parent: self.span.clone(), Level::INFO, "outbound")),
        )
    }
}
