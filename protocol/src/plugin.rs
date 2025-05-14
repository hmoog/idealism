use std::pin::Pin;

use tracing::Span;

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

    fn span(&self) -> Span;
}
