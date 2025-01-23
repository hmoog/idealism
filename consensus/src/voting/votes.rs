use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use crate::{ConfigInterface, Vote, VoteRefs};

pub struct Votes<T: ConfigInterface>(HashSet<Vote<T>>);

impl<T: ConfigInterface> Votes<T> {
    pub fn any_round(&self) -> u64 {
        self.0.iter().next().map(|vote| vote.round()).unwrap_or(0)
    }

    pub fn heaviest(&self, weights: &HashMap<Vote<T>, u64>) -> Option<Vote<T>> {
        self.iter()
            .map(|candidate_weak| {
                (candidate_weak.clone(), weights.get(candidate_weak).unwrap_or(&0))
            })
            .max_by(|(candidate1, weight1), (candidate2, weight2)| {
                weight1.cmp(weight2).then_with(|| candidate1.cmp(candidate2))
            })
            .map(|(candidate, _)| { candidate })
    }

    pub fn downgrade(&self) -> VoteRefs<T> {
        self.iter().map(|vote| vote.downgrade()).collect()
    }
}

impl<ID: ConfigInterface> Default for Votes<ID> {
    fn default() -> Self {
        Self(HashSet::default())
    }
}

impl<ID: ConfigInterface> FromIterator<Vote<ID>> for Votes<ID> {
    fn from_iter<I: IntoIterator<Item=Vote<ID>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<ID: ConfigInterface> Clone for Votes<ID> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
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