use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use crate::config::Config;
use crate::vote::Vote;

pub struct Votes<T: Config>(HashSet<Vote<T>>);

impl<T: Config> Votes<T> {
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

impl<ID: Config> Clone for Votes<ID> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<ID: Config> Default for Votes<ID> {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl<T: Config> IntoIterator for Votes<T> {
    type Item = Vote<T>;
    type IntoIter = std::collections::hash_set::IntoIter<Vote<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: Config> IntoIterator for &'a Votes<T> {
    type Item = &'a Vote<T>;
    type IntoIter = std::collections::hash_set::Iter<'a, Vote<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: Config> Deref for Votes<T> {
    type Target = HashSet<Vote<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Config> DerefMut for Votes<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}