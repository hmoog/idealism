use std::sync::Arc;

use crate::{Plugin, Plugins};

pub trait ManagedPlugin: Sized {
    fn new(plugins: &mut Plugins) -> Arc<Self>;

    fn start(&self) {
        // do nothing by default
    }

    fn shutdown(&self) {
        // do nothing by default
    }
}

impl<T: ManagedPlugin> Plugin for T {
    fn start(&self) {
        ManagedPlugin::start(self);
    }

    fn shutdown(&self) {
        ManagedPlugin::shutdown(self);
    }
}
