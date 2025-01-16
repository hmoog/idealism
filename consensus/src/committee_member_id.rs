use std::hash::Hash;

pub trait CommitteeMemberID: PartialEq + Eq + Hash + Default {}
impl<T: PartialEq + Eq + Hash + Default> CommitteeMemberID for T {}