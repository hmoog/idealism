use std::fmt::{Debug, Display};

pub enum Error {
    BlockDagErr(blockdag::Error),
    VoteFailed(virtual_voting::Error),
    UnsupportedBlockType,
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<blockdag::Error> for Error {
    fn from(error: blockdag::Error) -> Self {
        Error::BlockDagErr(error)
    }
}

impl From<virtual_voting::Error> for Error {
    fn from(error: virtual_voting::Error) -> Self {
        Error::VoteFailed(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BlockDagErr(err) => write!(f, "Block Dag error: {}", err),
            Error::VoteFailed(error) => write!(f, "Virtual voting error: {}", error),
            Error::UnsupportedBlockType => write!(f, "Unsupported block type"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
