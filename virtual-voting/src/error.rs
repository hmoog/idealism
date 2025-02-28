use std::fmt::Display;

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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ReferencedVoteEvicted => write!(f, "Referenced vote evicted"),
            Error::VotesMustNotBeEmpty => write!(f, "Votes must not be empty"),
            Error::NoAcceptedMilestoneInPastCone => write!(f, "No accepted milestone in past cone"),
            Error::NoConfirmedMilestoneInPastCone => {
                write!(f, "No confirmed milestone in past cone")
            }
            Error::NoCommitmentExists => write!(f, "No commitment exists"),
            Error::NoMilestone => write!(f, "No milestone"),
            Error::TimeMustIncrease => write!(f, "Time must increase"),
        }
    }
}
