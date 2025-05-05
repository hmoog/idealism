use std::pin::Pin;

pub trait Plugin: Send + Sync {
    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        None
    }

    fn shutdown(&self) {}
}
