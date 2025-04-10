use std::any::TypeId;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Block not found")]
    BlockNotFound,

    #[error("Metadata not found")]
    MetadataNotFound(TypeId),
}

pub type BlockDAGResult<T> = Result<T, Error>;
