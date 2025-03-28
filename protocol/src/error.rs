use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Block Dag error: {0}")]
    BlockDagErr(#[from] blockdag::Error),

    #[error("Virtual voting error: {0}")]
    VoteFailed(#[from] virtual_voting::Error),

    #[error("Unsupported block type")]
    UnsupportedBlockType,
}

pub type Result<T> = core::result::Result<T, Error>;
