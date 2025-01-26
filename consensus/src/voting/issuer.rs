use std::hash::{Hash, Hasher};

use utils::ArcKey;

use crate::CommitteeMemberID;

#[derive(Clone)]
pub enum Issuer<ID: CommitteeMemberID> {
    System,
    User(ArcKey<ID>),
}

impl<T: CommitteeMemberID> Hash for Issuer<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Issuer::System => 0.hash(state),
            Issuer::User(issuer) => {
                1.hash(state);
                issuer.hash(state)
            }
        }
    }
}

impl<T: CommitteeMemberID> PartialEq for Issuer<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Issuer::System, Issuer::System) => true,
            (Issuer::User(issuer1), Issuer::User(issuer2)) => *issuer1 == *issuer2,
            _ => false,
        }
    }
}

impl<T: CommitteeMemberID> Eq for Issuer<T> {}
