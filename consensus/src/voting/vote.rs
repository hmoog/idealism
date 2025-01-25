use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::Arc,
};

use newtype::define;
use utils::{ArcKey, rx};

use crate::{
    ConfigInterface, VoteData, VoteRef, VoteRefs, VotesByIssuer,
    errors::{Error, Error::ReferencedVoteEvicted},
};

define!(Vote, Arc<VoteData<T>>, T: ConfigInterface);

impl<ID: ConfigInterface> Vote<ID> {
    pub fn new_genesis(config: ID) -> Vote<ID> {
        Vote(Arc::new_cyclic(|me| {
            let committee = config.select_committee(None);

            VoteData {
                accepted: rx::Signal::new().init(true),
                cumulative_slot_weight: 0,
                round: 0,
                leader_weight: 0,
                issuer: ArcKey::new(ID::CommitteeMemberID::default()),
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

    pub fn aggregate(
        issuing_identity: &ArcKey<ID::CommitteeMemberID>,
        votes: Vec<&Vote<ID>>,
    ) -> Result<Vote<ID>, Error> {
        let mut heaviest_vote = *votes.first().ok_or(Error::VotesMustNotBeEmpty)?;
        let mut votes_by_issuer: VotesByIssuer<ID> = VotesByIssuer::default();
        for vote in votes {
            votes_by_issuer.collect_from(&vote.votes_by_issuer().upgrade()?);

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

impl<ID: ConfigInterface> TryFrom<VoteRef<ID>> for Vote<ID> {
    type Error = Error;

    fn try_from(vote_ref: VoteRef<ID>) -> Result<Self, Self::Error> {
        vote_ref
            .upgrade()
            .map(Vote::new)
            .ok_or(ReferencedVoteEvicted)
    }
}

impl<ID: ConfigInterface> TryFrom<&VoteRef<ID>> for Vote<ID> {
    type Error = Error;

    fn try_from(vote_ref: &VoteRef<ID>) -> Result<Self, Self::Error> {
        vote_ref
            .upgrade()
            .map(Vote::new)
            .ok_or(ReferencedVoteEvicted)
    }
}

impl<ID: ConfigInterface> Ord for Vote<ID> {
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

impl<T: ConfigInterface> Hash for Vote<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Arc::as_ptr(&self.0).hash(hasher)
    }
}
