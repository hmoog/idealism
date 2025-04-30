mod config;

pub use config::ProtocolConfig;

use std::sync::Arc;

use common::plugins::Plugins;
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

