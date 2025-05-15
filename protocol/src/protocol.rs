use std::sync::Arc;

use tracing::info;

use crate::{Plugins, ProtocolConfig};

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
        info!("starting protocol");
        self.plugins.start().await;
        info!("stopped protocol");
    }

    pub fn shutdown(&self) {
        info!("shutting down protocol");
        self.plugins.shutdown();
    }
}
