use std::hash::Hash;

pub trait CommitteeMemberID: PartialEq + Eq + Hash {}
impl<T: PartialEq + Eq + Hash> CommitteeMemberID for T {}