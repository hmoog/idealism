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

    pub fn cumulative_slot_weight(&self) -> u64 {
        self.0.cumulative_slot_weight
    }

    pub fn round(&self) -> u64 {
        self.0.round
    }

    pub fn leader_weight(&self) -> u64 {
        self.0.leader_weight
    }

    pub fn downgrade(&self) -> VoteRef<ID> {
        (&self.0).into()
    }
}

impl<T: CommitteeMemberID> From<Arc<VoteData<T>>> for Vote<T> {
    fn from(arc: Arc<VoteData<T>>) -> Self {
        Self(arc)
    }
}