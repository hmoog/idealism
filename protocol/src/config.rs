use blockdag::BlockDAGConfig;
use common::plugins::PluginRegistry;
use virtual_voting::VirtualVotingConfig;

use crate::{ProtocolError, ProtocolPlugin};

pub trait ProtocolConfig: VirtualVotingConfig + BlockDAGConfig<ErrorType = ProtocolError> {
    fn inject_plugins(
        &self,
        registry: PluginRegistry<dyn ProtocolPlugin<Self>>,
    ) -> PluginRegistry<dyn ProtocolPlugin<Self>>;
}
