use std::fmt::{Debug, Display};

pub enum Error {
    VoteNotFound,
    BlockNotFound,
}

pub type Result<T> = core::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::VoteNotFound => write!(f, "Vote not found"),
            Error::BlockNotFound => write!(f, "Block not found"),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
