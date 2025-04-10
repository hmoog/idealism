use common::{ids::BlockID, plugins::PluginRegistry};

use crate::{BlockDAGPlugin, BlockMetadataRef};

pub trait BlockDAGConfig:
    virtual_voting::VirtualVotingConfig<Source = BlockMetadataRef<Self>>
{
    type ErrorType: Send;

    fn genesis_block_id(&self) -> BlockID;

    fn inject_plugins(
        &self,
        registry: PluginRegistry<dyn BlockDAGPlugin<Self>>,
    ) -> PluginRegistry<dyn BlockDAGPlugin<Self>>;
}
