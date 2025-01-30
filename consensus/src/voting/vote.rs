use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::Arc,
};

use newtype::{CloneInner, DerefInner};
use utils::ArcKey;

use crate::{ConfigInterface, VoteData, VoteRef, VoteRefs, VoteRefsByIssuer, Votes, errors::Error};

#[derive(CloneInner, DerefInner)]
pub struct Vote<Config: ConfigInterface>(Arc<VoteData<Config>>);

impl<Config: ConfigInterface> Vote<Config> {
    pub fn cast(
        issuer: &ArcKey<Config::CommitteeMemberID>,
        votes: Vec<&Vote<Config>>,
    ) -> Result<Vote<Config>, Error> {
        VoteData::try_from(Votes::from_iter(votes.into_iter().cloned()))?.build(issuer.clone())
    }
}

impl<Config: ConfigInterface> From<Arc<VoteData<Config>>> for Vote<Config> {
    fn from(arc: Arc<VoteData<Config>>) -> Self {
        Self(arc)
    }
}

impl<Config: ConfigInterface> From<Config> for Vote<Config> {
    fn from(config: Config) -> Self {
        Self(Arc::new_cyclic(|me| {
            let mut vote = VoteData::from(config);
            vote.target = me.into();
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
            .map(Vote::from)
            .ok_or(Error::ReferencedVoteEvicted)
    }
}

impl<Config: ConfigInterface> PartialEq<Self> for Vote<Config> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<Config: ConfigInterface> PartialOrd for Vote<Config> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Config: ConfigInterface> Eq for Vote<Config> {}

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
