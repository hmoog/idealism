use common::{
    plugins::{Plugin, PluginRegistry},
};

use crate::ProtocolPlugin;

pub trait ProtocolConfig:
    ProtocolPlugin + Plugin<dyn ProtocolPlugin> + Sync + Send + 'static
{
    fn inject_plugins(
        &self,
        registry: PluginRegistry<dyn ProtocolPlugin>,
    ) -> PluginRegistry<dyn ProtocolPlugin>;
}
