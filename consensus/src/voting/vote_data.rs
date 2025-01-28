use std::sync::{Arc, Weak};

use utils::ArcKey;

use crate::{
    Committee, ConfigInterface, Issuer, Vote, VoteRef, VoteRefs, VoteRefsByIssuer, Votes,
    VotesByIssuer, consensus::ConsensusRound, errors::Error,
};

pub struct VoteData<T: ConfigInterface> {
    pub config: Arc<T>,
    pub issuer: Issuer<T::CommitteeMemberID>,
    pub accepted: bool,
    pub cumulative_slot_weight: u64,
    pub round: u64,
    pub leader_weight: u64,
    pub committee: Committee<T>,
    pub votes_by_issuer: VoteRefsByIssuer<T>,
    pub target: VoteRef<T>,
}

impl<T: ConfigInterface> VoteData<T> {
    pub(crate) fn build(mut self, issuer: ArcKey<T::CommitteeMemberID>) -> Result<Vote<T>, Error> {
        // TODO: HANDLE FROM CONFIG:
        // votes_by_issuer.retain(|id, _| heaviest_tip.committee.is_member_online(id));

        self.issuer = Issuer::User(issuer.clone());

        // abort if the issuer is not a member of the committee
        let Some(committee_member) = self.committee.member(&issuer).cloned() else {
            return Ok(Vote::new(Arc::new(self)));
        };

        // set the issuer online if they are not already
        if !committee_member.is_online() {
            self.committee = self.committee.set_online(&issuer, true);
        }

        // determine the acceptance threshold
        let referenced_round_weight = self
            .committee
            .referenced_round_weight(&self.votes_by_issuer)?;
        let acceptance_threshold = self.committee.acceptance_threshold();

        // abort if we have already voted and are below the acceptance threshold
        let own_votes = self.votes_by_issuer.entry(issuer.clone()).or_default();
        if let Some(own_vote) = own_votes.iter().next() {
            let vote: Vote<T> = own_vote.try_into()?;
            if vote.round == self.round && referenced_round_weight < acceptance_threshold {
                return Ok(Vote::new(Arc::new(self)));
            }
        }

        // determine the target vote
        let mut consensus_round = ConsensusRound::new(self.committee.clone());
        let latest_accepted_milestone = consensus_round
            .latest_accepted_milestone(VotesByIssuer::try_from(&self.votes_by_issuer)?.into())?;
        self.target =
            VoteRef::from(&consensus_round.heaviest_descendant(&latest_accepted_milestone));

        // advance the round if the acceptance threshold is now met
        if referenced_round_weight + committee_member.weight() >= acceptance_threshold {
            self.leader_weight = self.config.leader_weight(&self);
            self.round += 1;
        }

        Ok(Vote::new(Arc::new_cyclic(|me| {
            self.votes_by_issuer
                .insert(issuer, VoteRefs::from_iter([VoteRef::new(me.clone())]));
            self
        })))
    }
}

impl<Config: ConfigInterface> TryFrom<Votes<Config>> for VoteData<Config> {
    type Error = Error;
    fn try_from(votes: Votes<Config>) -> Result<VoteData<Config>, Self::Error> {
        let heaviest_tip = votes.heaviest().clone().expect("votes must not be empty");

        Ok(VoteData {
            issuer: Issuer::System,
            votes_by_issuer: VotesByIssuer::try_from(votes)?.into(),
            committee: heaviest_tip.committee.clone(),
            config: heaviest_tip.config.clone(),
            target: heaviest_tip.target.clone(),
            cumulative_slot_weight: heaviest_tip.cumulative_slot_weight,
            round: heaviest_tip.round,
            leader_weight: heaviest_tip.leader_weight,
            accepted: false,
        })
    }
}

impl<Config: ConfigInterface> From<Config> for VoteData<Config> {
    fn from(config: Config) -> Self {
        Self {
            issuer: Issuer::System,
            votes_by_issuer: VoteRefsByIssuer::default(),
            committee: config.select_committee(None),
            config: Arc::new(config),
            target: VoteRef::new(Weak::new()),
            cumulative_slot_weight: 0,
            round: 0,
            leader_weight: 0,
            accepted: true,
        }
    }
}
