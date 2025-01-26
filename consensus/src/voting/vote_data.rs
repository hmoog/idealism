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
    pub(crate) fn build(
        mut self,
        issuer: ArcKey<T::CommitteeMemberID>,
    ) -> Result<Arc<Self>, Error> {
        self.issuer = Issuer::User(issuer.clone());

        // abort if the issuer is not a member of the committee
        let Some(committee_member) = self.committee.member(&issuer).cloned() else {
            return Ok(Arc::new(self));
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
                return Ok(Arc::new(self));
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

        Ok(Arc::new_cyclic(|me| {
            self.votes_by_issuer
                .insert(issuer, VoteRefs::from_iter([VoteRef::new(me.clone())]));
            self
        }))
    }
}

impl<Config: ConfigInterface> TryFrom<Votes<Config>> for VoteData<Config> {
    type Error = Error;
    fn try_from(votes: Votes<Config>) -> Result<VoteData<Config>, Self::Error> {
        let mut heaviest_vote = votes.iter().next().expect("votes mst not be empty").clone();
        let mut votes_by_issuer: VotesByIssuer<Config> = VotesByIssuer::default();
        for vote in votes {
            votes_by_issuer.collect_from(&VotesByIssuer::try_from(&vote.votes_by_issuer)?);

            if vote > heaviest_vote {
                heaviest_vote = vote;
            }
        }
        let committee = heaviest_vote.committee.clone();

        // for all online committee members (check if they are still online and retain
        // only their votes)

        votes_by_issuer.retain(|id, _| committee.is_member_online(id));

        Ok(VoteData {
            config: heaviest_vote.config.clone(),
            accepted: false,
            cumulative_slot_weight: heaviest_vote.cumulative_slot_weight,
            round: heaviest_vote.round,
            leader_weight: heaviest_vote.leader_weight,
            issuer: Issuer::System,
            committee,
            votes_by_issuer: votes_by_issuer.downgrade(),
            target: heaviest_vote.target.clone(),
        })
    }
}

impl<Config: ConfigInterface> From<Config> for VoteData<Config> {
    fn from(config: Config) -> Self {
        Self {
            cumulative_slot_weight: 0,
            round: 0,
            leader_weight: 0,
            issuer: Issuer::System,
            votes_by_issuer: VoteRefsByIssuer::default(),
            target: VoteRef::new(Weak::new()),
            committee: config.select_committee(None),
            config: Arc::new(config),
            accepted: true,
        }
    }
}
