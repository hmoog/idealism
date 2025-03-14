use std::fmt::Display;

pub enum Error {
    VoteNotFound,
    VoteFailed(virtual_voting::Error),
    BlockNotFound,
    UnsupportedBlockType,
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<virtual_voting::Error> for Error {
    fn from(error: virtual_voting::Error) -> Self {
        Error::VoteFailed(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::VoteNotFound => write!(f, "Vote not found"),
            Error::VoteFailed(error) => write!(f, "Virtual voting error: {}", error),
            Error::BlockNotFound => write!(f, "Block not found"),
            Error::UnsupportedBlockType => write!(f, "Unsupported block type"),
        }
    }
}
