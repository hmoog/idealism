mod config;
mod managed_plugin;
mod plugin;
mod plugins;
use std::sync::Arc;

pub use config::ProtocolConfig;
pub use managed_plugin::ManagedPlugin;
pub use plugin::Plugin;
pub use plugins::Plugins;
use zero::{Clone0, Deref0};

#[derive(Deref0, Clone0)]
pub struct Protocol(Arc<ProtocolData>);

pub struct ProtocolData {
    pub plugins: Plugins,
}

impl Protocol {
    pub fn new(config: impl ProtocolConfig) -> Self {
        let mut plugins = Plugins::default();

        Self(Arc::new(ProtocolData {
            plugins: ProtocolConfig::inject_plugins(&*plugins.provide(Arc::new(config)), plugins),
        }))
    }
}
