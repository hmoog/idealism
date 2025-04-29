pub trait ProtocolPlugin: Send + Sync {
    fn shutdown(&self);
}
