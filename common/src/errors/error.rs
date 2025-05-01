use thiserror::Error;

use crate::ids::BlockID;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Block not found")]
    BlockNotFound,

    #[error("Metadata not found for type `{metadata}`")]
    MetadataNotFound {
        block_id: BlockID,
        metadata: &'static str,
    },
}
