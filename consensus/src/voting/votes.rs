use std::collections::{HashMap, HashSet};

use crate::{ConfigInterface, Vote, VoteRefs, errors::Error};

pub struct Votes<Config: ConfigInterface> {
    elements: HashSet<Vote<Config>>,
    max: Option<Vote<Config>>,
}

impl<Config: ConfigInterface> Votes<Config> {
    pub fn max(&self) -> &Option<Vote<Config>> {
        &self.max
    }

    pub fn insert(&mut self, vote: Vote<Config>) -> bool {
        if self.max.as_ref().map_or(true, |v| vote > *v) {
            self.max = Some(vote.clone());
        }

        self.elements.insert(vote.clone())
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<Vote<Config>> {
        self.elements.iter()
    }

    pub fn heaviest(&self, weights: &HashMap<Vote<Config>, u64>) -> Option<Vote<Config>> {
        self.iter()
            .map(|candidate_weak| {
                (
                    candidate_weak.clone(),
                    weights.get(candidate_weak).unwrap_or(&0),
                )
            })
            .max_by(|(candidate1, weight1), (candidate2, weight2)| {
                weight1
                    .cmp(weight2)
                    .then_with(|| candidate1.cmp(candidate2))
            })
            .map(|(candidate, _)| candidate)
    }
}

impl<Config: ConfigInterface, U: Into<Vote<Config>>> FromIterator<U> for Votes<Config> {
    fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
        Self {
            elements: iter.into_iter().map(Into::into).collect(),
            max: None,
        }
    }
}

impl<C: ConfigInterface> TryFrom<VoteRefs<C>> for Votes<C> {
    type Error = Error;
    fn try_from(vote_refs: VoteRefs<C>) -> Result<Self, Self::Error> {
        vote_refs
            .into_inner()
            .into_iter()
            .map(Vote::try_from)
            .collect()
    }
}

impl<C: ConfigInterface> TryFrom<&VoteRefs<C>> for Votes<C> {
    type Error = Error;
    fn try_from(vote_refs: &VoteRefs<C>) -> Result<Self, Self::Error> {
        vote_refs.iter().map(Vote::try_from).collect()
    }
}

impl<Config: ConfigInterface> Extend<Vote<Config>> for Votes<Config> {
    fn extend<T: IntoIterator<Item = Vote<Config>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|v| {
            self.insert(v);
        });
    }
}

impl<Config: ConfigInterface> IntoIterator for Votes<Config> {
    type Item = Vote<Config>;
    type IntoIter = std::collections::hash_set::IntoIter<Vote<Config>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a, Config: ConfigInterface> IntoIterator for &'a Votes<Config> {
    type Item = &'a Vote<Config>;
    type IntoIter = std::collections::hash_set::Iter<'a, Vote<Config>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<Config: ConfigInterface> Clone for Votes<Config> {
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
            max: self.max.clone(),
        }
    }
}

impl<Config: ConfigInterface> Default for Votes<Config> {
    fn default() -> Self {
        Self {
            elements: HashSet::new(),
            max: None,
        }
    }
}
