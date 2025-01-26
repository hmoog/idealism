use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::Arc,
};

use utils::ArcKey;

use crate::{ConfigInterface, VoteData, VoteRef, VoteRefs, VoteRefsByIssuer, Votes, errors::Error};

newtype::define!(Vote, Arc<VoteData<Config>>, Config: ConfigInterface);

impl<Config: ConfigInterface> Vote<Config> {
    pub fn cast(
        issuer: &ArcKey<Config::CommitteeMemberID>,
        votes: Vec<&Vote<Config>>,
    ) -> Result<Vote<Config>, Error> {
        Ok(Vote(
            VoteData::try_from(Votes::from_iter(votes.into_iter().cloned()))?
                .build(issuer.clone())?,
        ))
    }
}

impl<Config: ConfigInterface> From<Config> for Vote<Config> {
    fn from(config: Config) -> Self {
        Self(Arc::new_cyclic(|me| {
            let mut vote = VoteData::from(config);
            vote.target = VoteRef::new(me.clone());
            vote.votes_by_issuer =
                VoteRefsByIssuer::from_iter(vote.committee.iter().map(|member| {
                    (
                        member.key().clone(),
                        VoteRefs::from_iter([vote.target.clone()]),
                    )
                }));
            vote
        }))
    }
}

impl<Config: ConfigInterface> TryFrom<VoteRef<Config>> for Vote<Config> {
    type Error = Error;
    fn try_from(vote_ref: VoteRef<Config>) -> Result<Self, Self::Error> {
        Vote::try_from(&vote_ref)
    }
}

impl<Config: ConfigInterface> TryFrom<&VoteRef<Config>> for Vote<Config> {
    type Error = Error;
    fn try_from(vote_ref: &VoteRef<Config>) -> Result<Self, Self::Error> {
        vote_ref
            .upgrade()
            .map(Vote::new)
            .ok_or(Error::ReferencedVoteEvicted)
    }
}

impl<Config: ConfigInterface> Ord for Vote<Config> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_weight = (self.cumulative_slot_weight, self.round, self.leader_weight);
        let other_weight = (
            other.cumulative_slot_weight,
            other.round,
            other.leader_weight,
        );

        self_weight.cmp(&other_weight)
    }
}

impl<Config: ConfigInterface> Hash for Vote<Config> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Arc::as_ptr(&self.0).hash(hasher)
    }
}
