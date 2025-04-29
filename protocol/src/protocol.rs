use std::sync::Arc;

use common::plugins::PluginRegistry;
use zero::{Clone0, Deref0};

use crate::{ProtocolConfig, ProtocolPlugin};

#[derive(Deref0, Clone0)]
pub struct Protocol<C: ProtocolConfig>(Arc<ProtocolData<C>>);

pub struct ProtocolData<C: ProtocolConfig> {
    pub plugins: PluginRegistry<dyn ProtocolPlugin<C>>,
    pub config: Arc<C>,
}

impl<C: ProtocolConfig> Protocol<C> {
    pub fn new(config: C) -> Self {
        Self(Arc::new(ProtocolData {
            plugins: ProtocolConfig::inject_plugins(&config, PluginRegistry::default()),
            config: Arc::new(config),
        }))
    }
}
