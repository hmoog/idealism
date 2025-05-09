use std::sync::Weak;

use common::blocks::Block;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{Network, NetworkID};

pub struct Endpoint {
    pub network: Weak<Network>,
    pub network_id: NetworkID,
    pub receiver: UnboundedReceiver<Block>,
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        if let Some(network) = self.network.upgrade() {
            network
                .participants
                .lock()
                .unwrap()
                .remove(&self.network_id);
        }
    }
}
