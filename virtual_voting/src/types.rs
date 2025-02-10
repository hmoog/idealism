use committee::MemberID;
use utils::Id;

#[derive(Debug)]
pub enum Error {
    ReferencedVoteEvicted,
    VotesMustNotBeEmpty,
    NoAcceptedMilestoneInPastCone,
    NoConfirmedMilestoneInPastCone,
    NoCommitmentExists,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Issuer<I: MemberID> {
    Genesis,
    User(Id<I>),
}
