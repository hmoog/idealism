use std::{any::Any, sync::Arc};

use crate::{Plugin, Plugins};

pub trait ProtocolConfig: Plugin + Sized + Sync + Send + 'static {
    fn with_params<T: Any + Send + Sync + 'static>(self, params: T) -> Self;

    fn params<T: Any + Send + Sync + 'static>(&self) -> Option<Arc<T>>;

    fn inject_plugins(&self, registry: Plugins) -> Plugins;
}
