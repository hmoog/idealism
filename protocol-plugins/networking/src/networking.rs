use std::sync::{Arc};
use tokio::sync::mpsc::{UnboundedReceiver};
use tokio::sync::{watch, RwLock};
use tokio::task::JoinHandle;
use common::blocks::Block;
use common::with;
use inbox::Inbox;
use outbox::Outbox;
use protocol::{ManagedPlugin, Plugins};

pub struct Endpoint {
    pub outbound: Network,
    pub inbound: UnboundedReceiver<Block>,
}

pub struct Networking {
    inbox: Arc<Inbox>,
    outbox: Arc<Outbox>,
    worker_details: RwLock<Option<(JoinHandle<()>, watch::Sender<()>)>>,
}

impl ManagedPlugin for Networking {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new(Self {
            inbox: plugins.load(),
            outbox: plugins.load(),
            worker_details: RwLock::new(None),
        })
    }
}

impl Networking {
    pub async fn connect(&self, endpoint: Endpoint) {
        let Endpoint { mut inbound, outbound } = endpoint;

        let mut worker_details = self.worker_details.write().await;

        // await for any previous task to finish
        if let Some((running_task, _)) = worker_details.take() {
            let _ = running_task.await;
        }

        let (shutdown_signal, is_shutdown) = watch::channel(());

        // create a new task for inbound messages
        let inbox = self.inbox.clone();
        let inbound_task = tokio::spawn(with!((mut is_shutdown): async move {
            loop {
                tokio::select! {
                    Some(block) = inbound.recv() => {
                        if let Err(e) = inbox.send(block) {
                            println!("Failed to receive block: {:?}", e);
                        }
                    },
                    _ = is_shutdown.changed() => break,
                }
            }
        }));

        // create a new task for outbound messages
        let outbox = self.outbox.clone();
        let outbound_task = tokio::spawn(with!((mut is_shutdown): async move {
            loop {
                let mut outbox = outbox.receiver.lock().await;
                tokio::select! {
                    Some(block) = outbox.recv() => {
                        if let Err(e) = outbound.send(block) {
                            println!("Failed to send block: {:?}", e);
                        }
                    },
                    _ = is_shutdown.changed() => break,
                }
            }
        }));

        *worker_details = Some((outbound_task, shutdown_signal));
    }
}

pub struct Worker {
    pub task: JoinHandle<()>,
    pub shutdown_signal: watch::Sender<()>,
}

pub struct Network {}

impl Network {
    pub fn send(&self, _block: Block) -> Result<(), tokio::sync::mpsc::error::SendError<Block>> {
        // Implement the logic to send the block to the network
        Ok(())
    }
}