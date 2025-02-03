use std::sync::Arc;

use utils::Id;

use crate::{
    Committee, ConfigInterface, ConsensusView, ConsensusViewRef, Issuer, Result, Vote, VoteRefs,
    VoteRefsByIssuer, Votes, VotesByIssuer,
};

pub struct VoteBuilder<T: ConfigInterface> {
    pub config: Arc<T>,
    pub issuer: Issuer<T::IssuerID>,
    pub cumulative_slot_weight: u64,
    pub round: u64,
    pub leader_weight: u64,
    pub committee: Committee<T>,
    pub votes_by_issuer: VoteRefsByIssuer<T>,
    pub consensus_view: ConsensusViewRef<T>,
}

impl<C: ConfigInterface> VoteBuilder<C> {
    pub fn new(votes: Votes<C>) -> Result<VoteBuilder<C>> {
        let heaviest_tip = votes
            .heaviest_element()
            .cloned()
            .expect("votes must not be empty");

        Ok(VoteBuilder {
            issuer: Issuer::Genesis,
            votes_by_issuer: VotesByIssuer::try_from(votes)?.into(),
            committee: heaviest_tip.committee.clone(),
            config: heaviest_tip.config.clone(),
            consensus_view: heaviest_tip.consensus_view.clone(),
            cumulative_slot_weight: heaviest_tip.cumulative_slot_weight,
            round: heaviest_tip.round,
            leader_weight: heaviest_tip.leader_weight,
        })
    }

    pub fn build(mut self, issuer: Id<C::IssuerID>) -> Result<Vote<C>> {
        // TODO: HANDLE FROM CONFIG:
        // votes_by_issuer.retain(|id, _| heaviest_tip.committee.is_member_online(id));

        self.issuer = Issuer::User(issuer.clone());

        // abort if the issuer is not a member of the committee
        let Some(committee_member) = self.committee.member(&issuer).cloned() else {
            return Ok(Vote::from(Arc::new(self)));
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

        // abort if we have already voted and are below the acceptance threshold to start a new
        // round
        let own_votes = self.votes_by_issuer.entry(issuer.clone()).or_default();
        if let Some(own_vote) = own_votes.iter().next() {
            let vote: Vote<C> = own_vote.try_into()?;
            if vote.round == self.round && referenced_round_weight < acceptance_threshold {
                return Ok(Vote::from(Arc::new(self)));
            }
        }

        self.consensus_view = ConsensusView::from_vote_builder(&self)?.into();

        // advance the round if the acceptance threshold is now met
        if referenced_round_weight + committee_member.weight() >= acceptance_threshold {
            self.leader_weight = self.config.leader_weight(&self);
            self.round += 1;
        }

        Ok(Vote::from(Arc::new_cyclic(|me| {
            self.votes_by_issuer
                .insert(issuer, VoteRefs::from_iter([me.into()]));
            self
        })))
    }

    pub fn build_genesis(config: C) -> Self {
        Self {
            issuer: Issuer::Genesis,
            votes_by_issuer: VoteRefsByIssuer::default(),
            committee: config.select_committee(None),
            config: Arc::new(config),
            consensus_view: ConsensusViewRef::default(),
            cumulative_slot_weight: 0,
            round: 0,
            leader_weight: 0,
        }
    }
}

mod traits {
    use crate::{ConfigInterface, Error, Result, VoteBuilder, Votes};

    impl<Config: ConfigInterface> TryFrom<Votes<Config>> for VoteBuilder<Config> {
        type Error = Error;
        fn try_from(votes: Votes<Config>) -> Result<VoteBuilder<Config>> {
            Self::new(votes)
        }
    }

    impl<Config: ConfigInterface> From<Config> for VoteBuilder<Config> {
        fn from(config: Config) -> Self {
            Self::build_genesis(config)
        }
    }
}
