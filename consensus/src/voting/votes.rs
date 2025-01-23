use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use crate::{ConfigInterface, Vote, VoteRefs};
use crate::errors::Error;

#[derive(Clone, Default)]
pub struct Votes<C: ConfigInterface>(HashSet<Vote<C>>);

impl<C: ConfigInterface> Votes<C> {
    pub fn any_round(&self) -> u64 {
        self.0.iter().next().map(|vote| vote.round()).unwrap_or(0)
    }

    pub fn heaviest(&self, weights: &HashMap<Vote<C>, u64>) -> Option<Vote<C>> {
        self.iter()
            .map(|candidate_weak| {
                (candidate_weak.clone(), weights.get(candidate_weak).unwrap_or(&0))
            })
            .max_by(|(candidate1, weight1), (candidate2, weight2)| {
                weight1.cmp(weight2).then_with(|| candidate1.cmp(candidate2))
            })
            .map(|(candidate, _)| { candidate })
    }
}

impl<C: ConfigInterface> TryFrom<&VoteRefs<C>> for Votes<C> {
    type Error = Error;

    fn try_from(vote_refs: &VoteRefs<C>) -> Result<Self, Self::Error> {
        vote_refs.iter().map(|v| v.upgrade().ok_or(Error::ReferencedVoteEvicted)).collect()
    }
}

impl<C: ConfigInterface> FromIterator<Vote<C>> for Votes<C> {
    fn from_iter<I: IntoIterator<Item=Vote<C>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<C: ConfigInterface> Deref for Votes<C> {
    type Target = HashSet<Vote<C>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: ConfigInterface> DerefMut for Votes<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}