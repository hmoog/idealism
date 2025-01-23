use std::sync::Arc;
use utils::{rx, ArcKey};
use crate::{Committee, ConfigInterface, VoteRef, VoteRefs, VoteRefsByIssuer};
use crate::consensus::ConsensusRound;
use crate::errors::Error;

pub struct VoteData<T: ConfigInterface> {
    pub(crate) config: Arc<T>,
    pub(crate) issuer: ArcKey<T::CommitteeMemberID>,
    pub(crate) accepted: rx::Signal<bool>,
    pub(crate) cumulative_slot_weight: u64,
    pub(crate) round: u64,
    pub(crate) leader_weight: u64,
    pub(crate) committee: Committee<T>,
    pub(crate) votes_by_issuer: VoteRefsByIssuer<T>,
    pub(crate) target: VoteRef<T>,
}

impl<T: ConfigInterface> VoteData<T> {
    pub fn issuer(&self) -> &ArcKey<T::CommitteeMemberID> {
        &self.issuer
    }

    pub fn committee(&self) -> &Committee<T> {
        &self.committee
    }

    pub fn votes_by_issuer(&self) -> &VoteRefsByIssuer<T> {
        &self.votes_by_issuer
    }

    pub fn cumulative_slot_weight(&self) -> u64 {
        self.cumulative_slot_weight
    }

    pub fn round(&self) -> u64 {
        self.round
    }

    pub fn leader_weight(&self) -> u64 {
        self.leader_weight
    }

    pub fn is_accepted(&self) -> bool {
        self.accepted.get().unwrap_or(false)
    }

    pub fn target(&self) -> &VoteRef<T> {
        &self.target
    }

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
            let own_round = own_vote.upgrade().ok_or(Error::ReferencedVoteEvicted)?.round();
            if own_round == self.round && referenced_round_weight < acceptance_threshold {
                return Ok(Arc::new(self));
            }
        }

        // determine the target vote
        let mut consensus_round = ConsensusRound::new(self.committee.clone());
        let latest_accepted_milestone = consensus_round.latest_accepted_milestone((&self.votes_by_issuer.upgrade()?).into())?;
        self.target = consensus_round.heaviest_descendant(&latest_accepted_milestone).downgrade();

        // advance the round if the acceptance threshold is now met
        if referenced_round_weight + committee_member.weight() >= acceptance_threshold {
            self.leader_weight = self.config.leader_weight(&self);
            self.round += 1;
        }

        Ok(Arc::new_cyclic(|me| {
            self.votes_by_issuer.insert(self.issuer.clone(), VoteRefs::new([VoteRef::new(me)]));
            self
        }))
    }
}