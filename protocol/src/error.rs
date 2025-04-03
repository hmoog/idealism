use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Block Dag error: {0}")]
    BlockDagError(#[from] blockdag::Error),

    #[error("Virtual voting error: {0}")]
    VirtualVotingError(#[from] virtual_voting::Error),
}

pub type ProtocolResult<T> = Result<T, ProtocolError>;
