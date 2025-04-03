use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Block Dag error: {0}")]
    BlockDagErr(#[from] blockdag::Error),

    #[error("Virtual voting error: {0}")]
    VoteFailed(#[from] virtual_voting::Error),

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Unsupported block type")]
    UnsupportedBlockType,
}

pub type ProtocolResult<T> = Result<T, ProtocolError>;
