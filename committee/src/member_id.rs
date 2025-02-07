use std::{fmt::Debug, hash::Hash};

pub trait MemberID: PartialEq + Eq + Hash + Default + Debug {}
impl<T: PartialEq + Eq + Hash + Default + Debug> MemberID for T {}
