use std::{pin::Pin, sync::Arc};

use crate::{Plugin, Plugins};

pub trait ManagedPlugin: Sized + Send + Sync {
    fn new(plugins: &mut Plugins) -> Arc<Self>;

    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        None
    }

    fn shutdown(&self) {
        // do nothing by default
    }
}

impl<T: ManagedPlugin> Plugin for T {
    fn start(&self) -> Option<Pin<Box<dyn Future<Output = ()> + Send>>> {
        ManagedPlugin::start(self)
    }

    fn shutdown(&self) {
        ManagedPlugin::shutdown(self);
    }
}
