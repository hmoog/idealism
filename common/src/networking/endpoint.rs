use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::blocks::Block;

pub struct Endpoint {
    pub inbound: UnboundedReceiver<Block>,
    pub outbound: UnboundedSender<Block>,
}
