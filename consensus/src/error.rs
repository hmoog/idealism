pub enum Error {
    VoteNotFound,
    VirtualVotingError(virtual_voting::Error),
}

pub type Result<T> = core::result::Result<T, Error>;

impl From<virtual_voting::Error> for Error {
    fn from(error: virtual_voting::Error) -> Self {
        Error::VirtualVotingError(error)
    }
}