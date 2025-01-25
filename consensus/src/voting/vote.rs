use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::Arc,
};

use newtype;
use utils::{ArcKey, rx};

use crate::{
    ConfigInterface, VoteData, VoteRef, VoteRefs, VotesByIssuer,
    errors::Error,
};

newtype::define!(Vote, Arc<VoteData<Config>>, Config: ConfigInterface);

impl<Config: ConfigInterface> Vote<Config> {
    pub fn cast(
        issuing_identity: &ArcKey<Config::CommitteeMemberID>,
        votes: Vec<&Vote<Config>>,
    ) -> Result<Vote<Config>, Error> {
        let mut heaviest_vote = *votes.first().ok_or(Error::VotesMustNotBeEmpty)?;
        let mut votes_by_issuer: VotesByIssuer<Config> = VotesByIssuer::default();
        for vote in votes {
            votes_by_issuer.collect_from(&vote.votes_by_issuer().try_into()?);

            if vote > heaviest_vote {
                heaviest_vote = vote;
            }
        }
        let committee = heaviest_vote.committee().clone();

        // for all online committee members (check if they are still online and retain
        // only their votes)

        votes_by_issuer.retain(|id, _| committee.is_member_online(id));

        Ok(Vote(
            VoteData {
                config: heaviest_vote.0.config.clone(),
                accepted: rx::Signal::new(),
                cumulative_slot_weight: heaviest_vote.cumulative_slot_weight(),
                round: heaviest_vote.round(),
                leader_weight: heaviest_vote.leader_weight(),
                issuer: issuing_identity.clone(),
                committee,
                votes_by_issuer: votes_by_issuer.downgrade(),
                target: heaviest_vote.target().clone(),
            }
            .build()?,
        ))
    }
}

impl<Config: ConfigInterface> From<Config> for Vote<Config> {
    fn from(config: Config) -> Self {
        Self(Arc::new_cyclic(|me| {
            let committee = config.select_committee(None);

            VoteData {
                accepted: rx::Signal::new().init(true),
                cumulative_slot_weight: 0,
                round: 0,
                leader_weight: 0,
                issuer: ArcKey::new(Config::CommitteeMemberID::default()),
                votes_by_issuer: committee
                    .iter()
                    .map(|member| {
                        (
                            member.key().clone(),
                            VoteRefs::from_iter([VoteRef::new(me.clone())]),
                        )
                    })
                    .collect(),
                target: VoteRef::new(me.clone()),
                config: Arc::new(config),
                committee,
            }
        }))
    }
}

impl<Config: ConfigInterface> TryFrom<VoteRef<Config>> for Vote<Config> {
    type Error = Error;
    fn try_from(vote_ref: VoteRef<Config>) -> Result<Self, Self::Error> {
        vote_ref.upgrade().map(Vote::new).ok_or(Error::ReferencedVoteEvicted)
    }
}

impl<Config: ConfigInterface> TryFrom<&VoteRef<Config>> for Vote<Config> {
    type Error = Error;
    fn try_from(vote_ref: &VoteRef<Config>) -> Result<Self, Self::Error> {
        vote_ref.upgrade().map(Vote::new).ok_or(Error::ReferencedVoteEvicted)
    }
}

impl<Config: ConfigInterface> Ord for Vote<Config> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_weight = (
            self.0.cumulative_slot_weight,
            self.0.round,
            self.0.leader_weight,
        );
        let other_weight = (
            other.0.cumulative_slot_weight,
            other.0.round,
            other.0.leader_weight,
        );

        self_weight.cmp(&other_weight)
    }
}

impl<Config: ConfigInterface> Hash for Vote<Config> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Arc::as_ptr(&self.0).hash(hasher)
    }
}
