use std::{cmp::Ordering, collections::HashSet, sync::Arc};

use committee::Committee;
use utils::Id;

use crate::{
    ConfigInterface, Error, Issuer, Milestone, Result, VirtualVoting, Vote, VoteRefs,
    VoteRefsByIssuer, Votes, VotesByIssuer,
};

pub struct VoteBuilder<T: ConfigInterface> {
    pub config: Arc<T>,
    pub issuer: Issuer<T::IssuerID>,
    pub time: u64,
    pub cumulative_slot_weight: u64,
    pub round: u64,
    pub committee: Committee<T::IssuerID>,
    pub referenced_milestones: VoteRefsByIssuer<T>,
    pub milestone: Option<Milestone<T>>,
}

impl<C: ConfigInterface> VoteBuilder<C> {
    pub fn new(votes: Votes<C>) -> Result<VoteBuilder<C>> {
        let heaviest_tip = votes
            .heaviest_element()
            .cloned()
            .expect("votes must not be empty");

        Ok(VoteBuilder {
            issuer: Issuer::Genesis,
            time: heaviest_tip.time,
            committee: heaviest_tip.committee.clone(),
            config: heaviest_tip.config.clone(),
            cumulative_slot_weight: heaviest_tip.cumulative_slot_weight,
            round: heaviest_tip.round,
            referenced_milestones: VotesByIssuer::try_from(votes)?.into(),
            milestone: None,
        })
    }

    pub fn milestone(&self) -> Result<&Milestone<C>> {
        self.milestone.as_ref().ok_or(Error::NoCommitmentExists)
    }

    pub fn build(mut self, issuer: &Id<C::IssuerID>, time: u64) -> Result<Vote<C>> {
        // update issuer
        self.issuer = Issuer::User(issuer.clone());

        // update time and flag offline validators
        if let Some(new_slot) = self.update_time(time) {
            self.flag_offline_validators(new_slot)?;
        }

        // check if we are a committee member
        if let Some(validator) = self.committee.member(issuer).cloned() {
            // set ourselves online before voting to also consider our weight
            self.committee = self.committee.set_online(validator.key(), true);

            // determine consensus threshold (switch between confirmation and acceptance)
            let (threshold, does_confirm) = self.consensus_threshold();

            // check if we shall vote
            if let Some(seen_weights) = self.shall_vote(validator.key(), threshold)? {
                // run virtual voting algorithm
                let (new_milestone, heaviest_tip) = VirtualVoting::run(&self, threshold)?;

                // update consensus weights
                if new_milestone.slot() > Vote::try_from(&new_milestone.milestone()?.prev)?.slot() {
                    self.cumulative_slot_weight += heaviest_tip.committee.online_weight();
                }
                if seen_weights + validator.weight() >= threshold {
                    self.round += 1;
                }

                self.milestone = Some(Milestone {
                    round_weight: self.config.leader_weight(&self),
                    prev: (&heaviest_tip).into(),
                    accepted: (&new_milestone).into(),
                    confirmed: match does_confirm {
                        true => (&new_milestone).into(),
                        false => heaviest_tip.milestone()?.confirmed.clone(),
                    },
                });

                // build vote and update votes_by_issuer map
                return Ok(Vote::from(Arc::new_cyclic(|me| {
                    self.referenced_milestones
                        .insert(validator.key().clone(), VoteRefs::from_iter([me.into()]));
                    self
                })));
            }
        }

        Ok(Vote::from(Arc::new(self)))
    }

    pub fn build_genesis(config: C) -> Self {
        Self {
            issuer: Issuer::Genesis,
            time: config.genesis_time(),
            committee: config.select_committee(None),
            config: Arc::new(config),
            cumulative_slot_weight: 0,
            round: 0,
            referenced_milestones: VoteRefsByIssuer::default(),
            milestone: None,
        }
    }

    fn slot(&self) -> u64 {
        self.config.slot_oracle(&self)
    }

    fn consensus_threshold(&self) -> (u64, bool) {
        if self.committee.online_weight() >= self.committee.confirmation_threshold() {
            (self.committee.confirmation_threshold(), true)
        } else {
            (self.committee.acceptance_threshold(), false)
        }
    }

    fn shall_vote(
        &self,
        issuer: &Id<C::IssuerID>,
        consensus_threshold: u64,
    ) -> Result<Option<u64>> {
        let seen_weight = self.seen_weight()?;
        if let Some(own_votes) = self.referenced_milestones.get(issuer) {
            if let Some(own_vote) = own_votes.iter().next() {
                if Vote::try_from(own_vote)?.round == self.round
                    && seen_weight < consensus_threshold
                {
                    return Ok(None);
                }
            }
        }

        Ok(Some(seen_weight))
    }

    fn seen_weight(&self) -> Result<u64> {
        let mut latest_round = 0;
        let mut referenced_round_weight = 0;

        for (issuer, votes) in &self.referenced_milestones {
            if let Some(member) = self.committee.member(issuer) {
                if let Some(vote_ref) = votes.iter().next() {
                    let vote = Vote::try_from(vote_ref)?;
                    match vote.round.cmp(&latest_round) {
                        Ordering::Greater => {
                            latest_round = vote.round;
                            referenced_round_weight = member.weight();
                        }
                        Ordering::Equal => {
                            referenced_round_weight += member.weight();
                        }
                        Ordering::Less => continue,
                    }
                }
            }
        }

        Ok(referenced_round_weight)
    }

    fn update_time(&mut self, issuing_time: u64) -> Option<u64> {
        let old_slot = self.slot();
        self.time = issuing_time;
        let new_slot = self.slot();

        (new_slot > old_slot).then_some(new_slot)
    }

    fn flag_offline_validators(&mut self, slot: u64) -> Result<()> {
        for member in self.offline_validators(slot - self.config.offline_threshold())? {
            self.committee = self.committee.set_online(&member, false);
        }
        Ok(())
    }

    fn offline_validators(&self, slot: u64) -> Result<HashSet<Id<C::IssuerID>>> {
        let mut idle_members = HashSet::new();
        for member in self.committee.iter() {
            if member.is_online() && self.validator_appears_offline(member.key(), slot)? {
                idle_members.insert(member.key().clone());
            }
        }
        Ok(idle_members)
    }

    fn validator_appears_offline(
        &self,
        id: &Id<C::IssuerID>,
        offline_threshold: u64,
    ) -> Result<bool> {
        let mut is_offline = true;
        if let Some(votes) = self.referenced_milestones.get(id) {
            for vote in votes.iter() {
                if Vote::try_from(vote)?.slot() >= offline_threshold {
                    is_offline = false;
                }
            }
        };

        Ok(is_offline)
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
