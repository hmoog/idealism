use types::ids::BlockID;

use crate::BlockMetadataRef;

pub trait Config: virtual_voting::Config<Source = BlockMetadataRef<Self>> {
    type ErrorType: Send;

    fn genesis_block_id(&self) -> BlockID;
}
