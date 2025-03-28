use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Vote not found")]
    VoteNotFound,

    #[error("Block not found")]
    BlockNotFound,
}

pub type Result<T> = core::result::Result<T, Error>;
