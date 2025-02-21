use std::{fmt::Debug, hash::Hash};

pub trait MemberID: PartialEq + Eq + Hash + Default + Debug + Sync + Send + 'static {}
impl<T: PartialEq + Eq + Hash + Default + Debug + Sync + Send + 'static> MemberID for T {}
