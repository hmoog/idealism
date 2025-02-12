use committee::MemberID;
use utils::Id;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Issuer<I: MemberID> {
    Genesis,
    User(Id<I>),
}
