use types::ids::IssuerID;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Issuer {
    Genesis,
    User(IssuerID),
}

impl Clone for Issuer {
    fn clone(&self) -> Self {
        match self {
            Issuer::Genesis => Issuer::Genesis,
            Issuer::User(id) => Issuer::User(id.clone()),
        }
    }
}
