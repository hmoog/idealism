use committee::MemberID;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Issuer {
    Genesis,
    User(MemberID),
}

impl Clone for Issuer {
    fn clone(&self) -> Self {
        match self {
            Issuer::Genesis => Issuer::Genesis,
            Issuer::User(id) => Issuer::User(id.clone()),
        }
    }
}
