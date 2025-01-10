use std::cmp::Ordering;
use std::sync::Arc;
use utils::{rx, ArcKey};
use crate::committee::Committee;
use crate::committee_member_id::CommitteeMemberID;
use crate::vote_ref::VoteRef;
use crate::votes_by_issuer::VotesByIssuer;

pub struct Vote<T: CommitteeMemberID>(Arc<VoteData<T>>);

pub(crate) struct VoteData<T: CommitteeMemberID> {
    pub issuer: ArcKey<T>,
    pub accepted: rx::Signal<bool>,
    pub cumulative_slot_weight: u64,
    pub round: u64,
    pub leader_weight: u64,
    pub committee: Committee<T>,
    pub votes_by_issuer: VotesByIssuer<T>,
    pub target: VoteRef<T>,
}

impl<ID: CommitteeMemberID> Vote<ID> {
    pub fn new_genesis(id: ID, committee: Committee<ID>) -> Vote<ID> {
        Vote(Arc::new_cyclic(|me| {
            VoteData {
                accepted: rx::Signal::new().init(true),
                cumulative_slot_weight: 0,
                round: 0,
                leader_weight: 0,
                issuer: ArcKey::new(id),
                votes_by_issuer: VotesByIssuer::new(),
                target: me.into(),
                committee,
            }
        }))
    }

    pub fn aggregate(issuing_identity: &ArcKey<ID>, votes: Vec<Vote<ID>>) -> Result<Vote<ID>, String> {
        let mut heaviest_vote = votes.first().ok_or("votes must not be empty")?
            .clone();
        let mut votes_by_issuer: VotesByIssuer<ID> = VotesByIssuer::new();
        for vote in votes {
            votes_by_issuer.collect_from(&vote.votes_by_issuer());

            if vote > heaviest_vote {
                heaviest_vote = vote;
            }
        }
        let committee = heaviest_vote.committee();

        // for all online committee members (check if they are still online and retain only their votes)

        votes_by_issuer.retain(|id, _| committee.is_member_online(id));

        Ok(Arc::new(VoteData {
            accepted: rx::Signal::new(),
            cumulative_slot_weight: heaviest_vote.cumulative_slot_weight(),
            round: heaviest_vote.round(),
            leader_weight: heaviest_vote.leader_weight(),
            issuer: issuing_identity.clone(),
            committee,
            votes_by_issuer,
            target: heaviest_vote.target(),
        }).into())
    }

    pub fn committee(&self) -> Committee<ID> {
        self.0.committee.clone()
    }

    pub fn votes_by_issuer(&self) -> &VotesByIssuer<ID> {
        &self.0.votes_by_issuer
    }

    pub fn cumulative_slot_weight(&self) -> u64 {
        self.0.cumulative_slot_weight
    }

    pub fn round(&self) -> u64 {
        self.0.round
    }

    pub fn leader_weight(&self) -> u64 {
        self.0.leader_weight
    }

    pub fn target(&self) -> VoteRef<ID> {
        self.0.target.clone()
    }

    pub fn downgrade(&self) -> VoteRef<ID> {
        (&self.0).into()
    }
}

impl<T: CommitteeMemberID> Clone for Vote<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<ID: CommitteeMemberID> Ord for Vote<ID> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_weight = (self.0.cumulative_slot_weight, self.0.round, self.0.leader_weight);
        let other_weight = (other.0.cumulative_slot_weight, other.0.round, other.0.leader_weight);

        self_weight.cmp(&other_weight)
    }
}

impl<ID: CommitteeMemberID> PartialOrd for Vote<ID> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<ID: CommitteeMemberID> PartialEq for Vote<ID> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<ID: CommitteeMemberID> Eq for Vote<ID> {}

impl<T: CommitteeMemberID> From<Arc<VoteData<T>>> for Vote<T> {
    fn from(arc: Arc<VoteData<T>>) -> Self {
        Self(arc)
    }
}