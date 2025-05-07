use std::pin::Pin;

pub trait Plugin: Send + Sync {
    fn plugin_name(&self) -> &'static str {
        std::any::type_name::<Self>()
            .split('<')
            .next()
            .unwrap()
            .rsplit("::")
            .next()
            .unwrap()
    }

    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        None
    }

    fn shutdown(&self) {}
}
