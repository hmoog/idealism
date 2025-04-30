use common::plugins::{Plugin, Plugins};

pub trait ProtocolConfig: Plugin + Sync + Send + 'static {
    fn inject_plugins(&self, registry: Plugins) -> Plugins;
}
