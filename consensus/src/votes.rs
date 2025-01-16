use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use crate::committee_member_id::CommitteeMemberID;
use crate::vote::Vote;

pub struct Votes<T: CommitteeMemberID>(HashSet<Vote<T>>);

impl<T: CommitteeMemberID> Votes<T> {
    pub fn new<const N: usize>(values: [Vote<T>; N]) -> Self {
        Votes(values.into_iter().collect())
    }

    pub fn first(&self) -> Option<Vote<T>> {
        self.0.iter().next().cloned()
    }

    pub fn any_round(&self) -> u64 {
        self.0.iter().next().map(|vote| vote.round()).unwrap_or(0)
    }
}

impl<ID: CommitteeMemberID> Clone for Votes<ID> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<ID: CommitteeMemberID> Default for Votes<ID> {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl<T: CommitteeMemberID> IntoIterator for Votes<T> {
    type Item = Vote<T>;
    type IntoIter = std::collections::hash_set::IntoIter<Vote<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CommitteeMemberID> IntoIterator for &'a Votes<T> {
    type Item = &'a Vote<T>;
    type IntoIter = std::collections::hash_set::Iter<'a, Vote<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: CommitteeMemberID> Deref for Votes<T> {
    type Target = HashSet<Vote<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: CommitteeMemberID> DerefMut for Votes<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}