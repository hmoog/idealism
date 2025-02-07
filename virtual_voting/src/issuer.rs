use committee::MemberID;
use utils::Id;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Issuer<I: MemberID> {
    Genesis,
    User(Id<I>),
}

impl<I: MemberID> Clone for Issuer<I> {
    fn clone(&self) -> Self {
        match self {
            Self::Genesis => Self::Genesis,
            Self::User(id) => Self::User(id.clone()),
        }
    }
}
