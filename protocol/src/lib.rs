mod config;
mod managed_plugin;
mod plugin;
mod plugins;

use std::sync::Arc;

pub use config::ProtocolConfig;
pub use managed_plugin::ManagedPlugin;
pub use plugin::Plugin;
pub use plugins::Plugins;
use tracing::info;

pub struct Protocol {
    pub plugins: Plugins,
}

impl Protocol {
    pub fn new(config: impl ProtocolConfig) -> Self {
        let mut plugins = Plugins::default();
        Self {
            plugins: ProtocolConfig::inject_plugins(&*plugins.provide(Arc::new(config)), plugins),
        }
    }

    pub async fn start(&self) {
        info!(target: "protocol", "started");
        self.plugins.start().await;
        info!(target: "protocol", "stopped");
    }

    pub fn shutdown(&self) {
        info!(target: "protocol", "shutting down");
        self.plugins.shutdown();
    }
}
