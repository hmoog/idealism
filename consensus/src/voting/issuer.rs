use utils::Id;

use crate::CommitteeMemberID;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Issuer<I: CommitteeMemberID> {
    Genesis,
    User(Id<I>),
}
