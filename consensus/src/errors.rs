#[derive(Debug)]
pub enum Error {
    ReferencedVoteEvicted,
    VotesMustNotBeEmpty,
    NoAcceptedMilestoneInPastCone,
    NoConfirmedMilestoneInPastCone,
}

pub type Result<T> = std::result::Result<T, Error>;
