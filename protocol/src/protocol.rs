use std::sync::Arc;

use common::plugins::PluginRegistry;
use zero::{Clone0, Deref0};

use crate::{ProtocolConfig, ProtocolPlugin};

#[derive(Deref0, Clone0)]
pub struct Protocol(Arc<ProtocolData>);

pub struct ProtocolData {
    pub plugins: PluginRegistry<dyn ProtocolPlugin>,
}

impl Protocol {
    pub fn new(config: impl ProtocolConfig) -> Self {
        let mut plugins = PluginRegistry::default();
        let config = plugins.set(Arc::new(config));

        Self(Arc::new(ProtocolData {
            plugins: ProtocolConfig::inject_plugins(&*config, plugins),
        }))
    }
}
