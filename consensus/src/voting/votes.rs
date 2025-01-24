use std::{
    collections::HashMap,
};
use newtype::define_hashset;
use crate::{ConfigInterface, Vote, VoteRefs, errors::Error};

define_hashset!(Votes, Vote<C>, C: ConfigInterface);

impl<C: ConfigInterface> Votes<C> {
    pub fn heaviest(&self, weights: &HashMap<Vote<C>, u64>) -> Option<Vote<C>> {
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

impl<C: ConfigInterface> TryFrom<VoteRefs<C>> for Votes<C> {
    type Error = Error;
    fn try_from(vote_refs: VoteRefs<C>) -> Result<Self, Self::Error> {
        vote_refs.0.into_iter().map(Vote::try_from).collect()
    }
}

impl<C: ConfigInterface> TryFrom<&VoteRefs<C>> for Votes<C> {
    type Error = Error;
    fn try_from(vote_refs: &VoteRefs<C>) -> Result<Self, Self::Error> {
        vote_refs.iter().map(Vote::try_from).collect()
    }
}
