use crate::ProtocolConfig;

pub trait ProtocolPlugin<C: ProtocolConfig>: Send + Sync {
    fn shutdown(&self);
}
