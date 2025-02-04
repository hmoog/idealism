use utils::Id;

use crate::CommitteeMemberID;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Issuer<I: CommitteeMemberID> {
    Genesis,
    User(Id<I>),
}

impl<I: CommitteeMemberID> Clone for Issuer<I> {
    fn clone(&self) -> Self {
        match self {
            Self::Genesis => Self::Genesis,
            Self::User(id) => Self::User(id.clone()),
        }
    }
}
