#[derive(Debug)]
pub enum Error {
    ReferencedVoteEvicted,
    VotesMustNotBeEmpty,
}

pub type Result<T> = std::result::Result<T, Error>;
