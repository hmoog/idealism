#[derive(Debug)]
pub enum Error {
    ReferencedVoteEvicted,
    VotesMustNotBeEmpty,
    NoAcceptedMilestoneInPastCone,
    NoConfirmedMilestoneInPastCone,
    NoCommitmentExists,
}

pub type Result<T> = std::result::Result<T, Error>;
