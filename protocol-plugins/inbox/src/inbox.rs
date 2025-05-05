use std::{
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
};

use block_storage::BlockStorage;
use common::{blocks::Block, down, extensions::ArcExt, up};
use protocol::{ManagedPlugin, Plugins};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task,
    task::JoinHandle,
};

pub struct Inbox {
    sender: RwLock<Option<UnboundedSender<Block>>>,
    receiver: Arc<Mutex<UnboundedReceiver<Block>>>,
    num_workers: usize,
    block_storage: Arc<BlockStorage>,
}

impl ManagedPlugin for Inbox {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        println!("Creating Inbox");
        let (tx, rx) = unbounded_channel();
        Arc::new(Self {
            sender: RwLock::new(Some(tx)),
            receiver: Arc::new(Mutex::new(rx)),
            num_workers: 2,
            block_storage: plugins.load(),
        })
    }

    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        let block_storage = self.block_storage.clone();

        let mut workers: Vec<JoinHandle<()>> = Vec::new();
        for i in 0..self.num_workers {
            let rx = Arc::clone(&self.receiver);

            workers.push(task::spawn_blocking(down!(block_storage: move || {
                println!("Worker {i} starting");
                loop {
                    up!(block_storage: match rx.lock().unwrap().blocking_recv() {
                        Some(block) => block_storage.insert(block),
                        None => break, // channel closed
                    });
                }
                println!("Worker {i} shutting down");
            })));
        }

        println!("Inbox is running with {} workers.", self.num_workers);

        Some(Box::pin(async move {
            for (i, handle) in workers.into_iter().enumerate() {
                match handle.await {
                    Ok(_) => println!("Worker {i} finished successfully."),
                    Err(e) => eprintln!("Worker {i} panicked: {e}"),
                }
            }
        }))
    }

    fn shutdown(&self) {
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
