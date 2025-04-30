use common::plugins::{Plugin, Plugins};

use crate::ProtocolPlugin;

pub trait ProtocolConfig: ProtocolPlugin + Plugin<dyn ProtocolPlugin> + Sync + Send + 'static
{
    fn inject_plugins(
        &self,
        registry: Plugins<dyn ProtocolPlugin>,
    ) -> Plugins<dyn ProtocolPlugin>;
}
