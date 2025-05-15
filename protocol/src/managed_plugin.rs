use std::{pin::Pin, sync::Arc};

use tracing::Span;

use crate::{Plugin, Plugins};

pub trait ManagedPlugin: Sized + Send + Sync {
    fn new(plugins: &mut Plugins) -> Arc<Self>;

    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        None
    }

    fn shutdown(&self) {
        // do nothing by default
    }

    fn span(&self) -> Span;
}

impl<T: ManagedPlugin> Plugin for T {
    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        ManagedPlugin::start(self)
    }

    fn shutdown(&self) {
        ManagedPlugin::shutdown(self);
    }

    fn span(&self) -> Span {
        ManagedPlugin::span(self)
    }
}
