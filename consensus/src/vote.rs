use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use utils::{rx, ArcKey};
use crate::committee::Committee;
use crate::committee_member_id::CommitteeMemberID;
use crate::consensus::ConsensusRound;
use crate::error::Error;
use crate::vote_ref::VoteRef;
use crate::vote_refs::VoteRefs;
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
    debug_alias: RwLock<Option<String>>
}

impl<T: CommitteeMemberID> VoteData<T> {
    pub(crate) fn build(mut self) -> Result<Arc<Self>, Error> {
        // abort if the issuer is not a member of the committee
        let Some(committee_member) = self.committee.member(&self.issuer).cloned() else {
            return Ok(Arc::new(self))
        };

        // set the issuer online if they are not already
        if !committee_member.is_online() {
            self.committee = self.committee.set_online(&self.issuer, true);
        }

        // determine the acceptance threshold
        let referenced_round_weight = self.committee.referenced_round_weight(&self.votes_by_issuer)?;
        let acceptance_threshold = self.committee.acceptance_threshold();

        // abort if we have already voted and are below the acceptance threshold
        let own_votes = self.votes_by_issuer.fetch(&self.issuer);
        if let Some(own_vote) = own_votes.first() {
            let own_round = own_vote.as_vote()?.round();
            if own_round == self.round && referenced_round_weight < acceptance_threshold {
                println!("aborting vote: own vote already cast and below acceptance threshold {} of {}", referenced_round_weight, acceptance_threshold);
                return Ok(Arc::new(self));
            }
        }

        // determine the target vote
        let mut consensus_round = ConsensusRound::new(self.committee.clone());
        let latest_accepted_milestone = consensus_round.latest_accepted_milestone((&self.votes_by_issuer).into());
        self.target = consensus_round.heaviest_descendant(&latest_accepted_milestone);

        // advance the round if the acceptance threshold is now met
        if referenced_round_weight + committee_member.weight() >= acceptance_threshold {
            self.round += 1;
            self.leader_weight = 0; // TODO: DETERMINE NEW LEADER WEIGHT FOR ROUND
        }

        Ok(Arc::new_cyclic(|me| {
            self.votes_by_issuer.insert(self.issuer.clone(), VoteRefs::new([me.into()]));
            self
        }))
    }
}

impl<ID: CommitteeMemberID> Vote<ID> {
    pub fn new_genesis(committee: Committee<ID>) -> Vote<ID> {
        Vote(Arc::new_cyclic(|me| {
            VoteData {
                accepted: rx::Signal::new().init(true),
                cumulative_slot_weight: 0,
                round: 0,
                leader_weight: 0,
                issuer: ArcKey::new(ID::default()),
                votes_by_issuer: committee
                    .iter()
                    .map(|member| (member.key().clone(), VoteRefs::new([me.into()])))
                    .collect::<VotesByIssuer<ID>>(),
                target: me.into(),
                committee,
                debug_alias: RwLock::new(None),
            }
        }))
    }

    pub fn aggregate(issuing_identity: &ArcKey<ID>, votes: Vec<&Vote<ID>>) -> Result<Vote<ID>, Error> {
        let mut heaviest_vote = *votes.first().ok_or(Error::VotesMustNotBeEmpty)?;
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

        Ok(Vote(VoteData {
            accepted: rx::Signal::new(),
            cumulative_slot_weight: heaviest_vote.cumulative_slot_weight(),
            round: heaviest_vote.round(),
            leader_weight: heaviest_vote.leader_weight(),
            issuer: issuing_identity.clone(),
            committee,
            votes_by_issuer,
            target: heaviest_vote.target().clone(),
            debug_alias: RwLock::new(None),
        }.build()?))
    }

    pub fn alias(&self) -> String {
        self.0.debug_alias.read().ok()
            .and_then(|alias| alias.clone())
            .unwrap_or_else(|| "<undefined>".to_string())
    }

    pub fn with_alias(self, alias: &str) -> Self {
        *self.0.debug_alias.write().unwrap() = Some(alias.to_string());
        self
    }

    pub fn ptr_eq(&self, other: &Vote<ID>) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn issuer(&self) -> &ArcKey<ID> {
        &self.0.issuer
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

    pub fn is_accepted(&self) -> bool {
        self.0.accepted.get().unwrap_or(false)
    }

    pub fn target(&self) -> &VoteRef<ID> {
        &self.0.target
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

impl<T: CommitteeMemberID> Hash for Vote<T> {
    fn hash<H : Hasher> (self: &'_ Self, hasher: &'_ mut H) {
        Arc::as_ptr(&self.0).hash(hasher)
    }
}