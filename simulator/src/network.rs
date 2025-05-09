use std::{
    collections::HashMap,
    sync::{Arc, Mutex, atomic::AtomicU64},
};

use common::{blocks::Block, extensions::ArcExt};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tracing::error;
use crate::{NetworkID, endpoint::Endpoint};

pub struct Network {
    pub participants: Mutex<HashMap<NetworkID, UnboundedSender<Block>>>,
    network_ids: AtomicU64,
}

impl Network {
    pub fn send(self: &Arc<Self>, network_id: NetworkID, block: Block) {
        if let Some(tx) = self.participants.lock().unwrap().get(&network_id) {
            if let Err(e) = tx.send(block) {
                error!(target: "network", "failed to send block to participant {}: {:?}", network_id, e);
            }
        } else {
            error!(target: "network", "participant {} not found", network_id);
        }
    }

    pub fn gossip(self: &Arc<Self>, source: NetworkID, block: Block) {
        for (network_id, tx) in self
            .participants
            .lock()
            .unwrap()
            .iter()
            .filter(|(id, _)| *id != &source)
        {
            if let Err(e) = tx.send(block.clone()) {
                error!(target: "network", "failed to send block to participant {}: {:?}", network_id, e);
            }
        }
    }

    pub fn new_endpoint(self: &Arc<Self>) -> Endpoint {
        let network_id = self
            .network_ids
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let (tx, rx) = unbounded_channel();

        self.participants.lock().unwrap().insert(network_id, tx);

        Endpoint {
            network: self.downgrade(),
            network_id,
            receiver: rx,
        }
    }
}
