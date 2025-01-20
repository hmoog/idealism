use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use crate::config::ConfigInterface;
use crate::voting::Vote;

pub struct Votes<T: ConfigInterface>(HashSet<Vote<T>>);

impl<T: ConfigInterface> Votes<T> {
    pub fn new<const N: usize>(values: [Vote<T>; N]) -> Self {
        Votes(values.into_iter().collect())
    }

    pub fn first(&self) -> Option<Vote<T>> {
        self.0.iter().next().cloned()
    }

    pub fn any_round(&self) -> u64 {
        self.0.iter().next().map(|vote| vote.round()).unwrap_or(0)
    }

    pub fn heaviest(&self, weights: &HashMap<Vote<T>, u64>) -> Option<Vote<T>> {
        self.iter()
            .filter_map(|candidate_weak| {
                Some((candidate_weak.clone(), weights.get(candidate_weak).unwrap_or(&0)))
            })
            .max_by(|(candidate1, weight1), (candidate2, weight2)| {
                weight1.cmp(weight2).then_with(|| candidate1.cmp(candidate2))
            })
            .map(|(candidate, _)| { candidate })
    }
}

impl<ID: ConfigInterface> Clone for Votes<ID> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<ID: ConfigInterface> Default for Votes<ID> {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl<T: ConfigInterface> IntoIterator for Votes<T> {
    type Item = Vote<T>;
    type IntoIter = std::collections::hash_set::IntoIter<Vote<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: ConfigInterface> IntoIterator for &'a Votes<T> {
    type Item = &'a Vote<T>;
    type IntoIter = std::collections::hash_set::Iter<'a, Vote<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<T: ConfigInterface> Deref for Votes<T> {
    type Target = HashSet<Vote<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ConfigInterface> DerefMut for Votes<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}