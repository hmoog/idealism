use common::ids::BlockID;

use crate::BlockMetadataRef;

pub trait BlockDAGConfig:
    virtual_voting::VirtualVotingConfig<Source = BlockMetadataRef<Self>>
{
    type ErrorType: Send;

    fn genesis_block_id(&self) -> BlockID;
}
