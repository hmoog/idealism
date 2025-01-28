use std::hash::Hash;

/// Trait bound for elements in [`Set`] requiring equality, hashing, ordering and cloning
/// capabilities.
pub trait SetElement: Eq + Hash + Ord + Clone {}

impl<T: Eq + Hash + Ord + Clone> SetElement for T {}
