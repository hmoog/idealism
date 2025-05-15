use async_trait::async_trait;
use tracing::Span;

#[async_trait]
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

    async fn start(&self) {}

    async fn shutdown(&self) {}

    fn span(&self) -> Span;
}
