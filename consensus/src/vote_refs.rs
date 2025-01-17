use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use crate::config::Config;
use crate::vote_ref::VoteRef;

pub struct VoteRefs<T: Config>(HashSet<VoteRef<T>>);

impl<T: Config> VoteRefs<T> {
    pub fn new<const N: usize>(values: [VoteRef<T>; N]) -> Self {
        VoteRefs(values.into_iter().collect())
    }

    pub fn first(&self) -> Option<VoteRef<T>> {
        self.0.iter().next().cloned()
    }

    pub fn any_round(&self) -> u64 {
        self.0.iter()
            .find_map(|vote_ref|
                vote_ref.upgrade().map(|vote| vote.round())
            )
            .unwrap_or(0)
    }
}

impl<ID: Config> Clone for VoteRefs<ID> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<ID: Config> Default for VoteRefs<ID> {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl<T: Config> IntoIterator for VoteRefs<T> {
    type Item = VoteRef<T>;
    type IntoIter = std::collections::hash_set::IntoIter<VoteRef<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: Config> IntoIterator for &'a VoteRefs<T> {
    type Item = &'a VoteRef<T>;
    type IntoIter = std::collections::hash_set::Iter<'a, VoteRef<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: Config> Deref for VoteRefs<T> {
    type Target = HashSet<VoteRef<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Config> DerefMut for VoteRefs<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}