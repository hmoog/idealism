#[derive(Debug)]
pub enum Error {
    ReferencedVoteEvicted,
    VotesMustNotBeEmpty,
    NoAcceptedMilestoneInPastCone,
    NoConfirmedMilestoneInPastCone,
    NoCommitmentExists,
    NoMilestone,
    TimeMustIncrease,
}

pub type Result<T> = std::result::Result<T, Error>;
