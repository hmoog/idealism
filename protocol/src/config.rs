use common::{blocks::BlockMetadataRef, plugins::PluginRegistry};
use virtual_voting::VirtualVotingConfig;

use crate::ProtocolPlugin;

pub trait ProtocolConfig: VirtualVotingConfig<Source = BlockMetadataRef> {
    fn inject_plugins(
        &self,
        registry: PluginRegistry<dyn ProtocolPlugin<Self>>,
    ) -> PluginRegistry<dyn ProtocolPlugin<Self>>;
}
