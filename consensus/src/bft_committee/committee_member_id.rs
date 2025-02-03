use std::{fmt::Debug, hash::Hash};

pub trait CommitteeMemberID: PartialEq + Eq + Hash + Default + Debug {}
impl<T: PartialEq + Eq + Hash + Default + Debug> CommitteeMemberID for T {}
