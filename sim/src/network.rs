use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::sync::Mutex;
use async_trait::async_trait;
use tracing::trace;
use common::blocks::Block;
use common::networking;
use common::networking::{Endpoint};

type NodeId = usize;

#[derive(Default)]
pub struct Network {
    next_id: AtomicUsize,
    nodes: Arc<Mutex<Vec<(NodeId, UnboundedSender<Block>)>>>,
}

#[async_trait]
impl networking::Network for Network {
    async fn endpoint(&self) -> Endpoint {
        let (tx_inbound, rx_inbound) = unbounded_channel::<Block>();
        let node_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        {
            let mut nodes = self.nodes.lock().await;
            nodes.push((node_id, tx_inbound.clone()));
        }

        let nodes = self.nodes.clone();

        let (tx_outbound, mut rx_outbound) = unbounded_channel::<Block>();
        tokio::spawn(async move {
            while let Some(block) = rx_outbound.recv().await {
                let nodes = nodes.lock().await;
                for (peer_id, peer_tx) in nodes.iter() {
                    if *peer_id != node_id {
                        trace!("Sending block {} from peer {} to peer {}", block.id(), node_id, peer_id);
                        let _ = peer_tx.send(block.clone()); // ignore send failures
                    }
                }
            }
        });

        Endpoint {
            inbound: rx_inbound,
            outbound: tx_outbound,
        }
    }
}
