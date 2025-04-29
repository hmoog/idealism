use common::{
    blocks::BlockMetadataRef,
    plugins::{Plugin, PluginRegistry},
};
use virtual_voting::VirtualVotingConfig;

use crate::ProtocolPlugin;

pub trait ProtocolConfig:
    ProtocolPlugin + Plugin<dyn ProtocolPlugin> + VirtualVotingConfig<Source = BlockMetadataRef>
{
    fn inject_plugins(
        &self,
        registry: PluginRegistry<dyn ProtocolPlugin>,
    ) -> PluginRegistry<dyn ProtocolPlugin>;
}
