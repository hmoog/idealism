use std::{sync::Arc};
use async_trait::async_trait;
use tracing::Span;

use crate::{Plugin, Plugins};

#[async_trait]
pub trait ManagedPlugin: Sized + Send + Sync {
    fn new(plugins: &mut Plugins) -> Arc<Self>;

    async fn start(&self) {
        // do nothing by default
    }

    async fn shutdown(&self) {
        // do nothing by default
    }

    fn span(&self) -> Span;
}

#[async_trait]
impl<T: ManagedPlugin> Plugin for T {
    async fn start(&self) {
        ManagedPlugin::start(self).await
    }

    async fn shutdown(&self) {
        ManagedPlugin::shutdown(self).await;
    }

    fn span(&self) -> Span {
        ManagedPlugin::span(self)
    }
}
