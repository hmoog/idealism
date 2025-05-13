use std::sync::{Arc};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{watch, Mutex};
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tracing::{error, trace};
use common::blocks::Block;
use common::networking::{Endpoint, Network};
use inbox::Inbox;
use outbox::Outbox;
use protocol::{ManagedPlugin, Plugins};

pub struct Networking {
    inbox: Arc<Inbox>,
    outbox: Arc<Outbox>,
    workers: Mutex<Option<(JoinHandle<()>, JoinHandle<()>, watch::Sender<()>)>>,
}

impl ManagedPlugin for Networking {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new(Self {
            inbox: plugins.load(),
            outbox: plugins.load(),
            workers: Mutex::new(None),
        })
    }
}

impl Networking {
    pub async fn connect<N: Network>(&self, network: &N) {
        let Endpoint { inbound, outbound } = network.endpoint().await;

        let mut workers = self.workers.lock().await;

        // wait for any previous workers to finish
        if let Some((inbound_worker, outbound_worker, shutdown)) = workers.take() {
            drop(shutdown); // close the shutdown channel to signal workers to stop

            // wait for workers to finish
            if let Err(e) = inbound_worker.await {
                error!(target: "networking", "inbound worker panicked: {:?}", e);
            }
            if let Err(e) = outbound_worker.await {
                error!(target: "networking", "outbound worker panicked: {:?}", e);
            }
        }

        // create new workers
        let (shutdown_signal, is_shutdown) = watch::channel(());
        *workers = Some((
            Self::new_inbound_worker(self.inbox.clone(), inbound, is_shutdown.clone()),
            Self::new_outbound_worker(self.outbox.clone(), outbound, is_shutdown.clone()),
            shutdown_signal,
        ));
    }

    pub async fn disconnect(&self) {
        // wait for any previous workers to finish
        if let Some((inbound_worker, outbound_worker, shutdown)) = self.workers.lock().await.take() {
            drop(shutdown); // close the shutdown channel to signal workers to stop

            // wait for workers to finish
            if let Err(e) = inbound_worker.await {
                error!(target: "networking", "inbound worker panicked: {:?}", e);
            }
            if let Err(e) = outbound_worker.await {
                error!(target: "networking", "outbound worker panicked: {:?}", e);
            }
        }
    }

    pub fn new_inbound_worker(inbox: Arc<Inbox>, mut receiver: UnboundedReceiver<Block>, mut is_shutdown: Receiver<()>) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(block) = receiver.recv() => {
                        if let Err(e) = inbox.send(block) {
                            error!(target: "networking", "failed to receive block: {:?}", e);
                        } else {
                            trace!(target: "networking", "received block");
                        }
                    },
                    _ = is_shutdown.changed() => break, // channel closed = shutdown
                }
            }
        })
    }

    pub fn new_outbound_worker(outbox: Arc<Outbox>, sender: UnboundedSender<Block>, mut is_shutdown: Receiver<()>) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut outbox = outbox.receiver.lock().await;
            loop {
                tokio::select! {
                    Some(block) = outbox.recv() => {
                        if let Err(e) = sender.send(block) {
                            error!(target: "networking", "failed to send block: {:?}", e);
                        } else {
                            trace!(target: "networking", "sent block");
                        }
                    },
                    _ = is_shutdown.changed() => break, // channel closed = shutdown
                }
            }
        })
    }
}
