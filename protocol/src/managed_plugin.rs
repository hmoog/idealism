use std::sync::Arc;

use crate::{Plugin, Plugins};

pub trait ManagedPlugin: Sized {
    fn construct(plugins: &mut Plugins) -> Arc<Self>;

    fn shutdown(&self);
}

impl<T: ManagedPlugin> Plugin for T {
    fn shutdown(&self) {
        ManagedPlugin::shutdown(self);
    }
}
