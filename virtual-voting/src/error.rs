use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Referenced vote evicted")]
    ReferencedVoteEvicted,

    #[error("Votes must not be empty")]
    VotesMustNotBeEmpty,

    #[error("No accepted milestone in past cone")]
    NoAcceptedMilestoneInPastCone,

    #[error("No confirmed milestone in past cone")]
    NoConfirmedMilestoneInPastCone,

    #[error("No commitment exists")]
    NoCommitmentExists,

    #[error("No milestone")]
    NoMilestone,

    #[error("Time must increase")]
    TimeMustIncrease,
}

pub type Result<T> = std::result::Result<T, Error>;
