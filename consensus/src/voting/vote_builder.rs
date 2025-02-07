use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::Arc;
use committee::{Committee, Member};
use utils::Id;

use crate::{ConfigInterface, ConsensusCommitment, ConsensusMechanism, Issuer, Result, Vote, VoteRefs, VoteRefsByIssuer, Votes, VotesByIssuer};

pub struct VoteBuilder<T: ConfigInterface> {
    pub config: Arc<T>,
    pub issuer: Issuer<T::IssuerID>,
    pub issuing_time: u64,
    pub cumulative_slot_weight: u64,
    pub round: u64,
    pub leader_weight: u64,
    pub committee: Committee<T::IssuerID>,
    pub votes_by_issuer: VoteRefsByIssuer<T>,
    pub consensus: ConsensusCommitment<T>,
}

impl<C: ConfigInterface> VoteBuilder<C> {
    pub fn new(votes: Votes<C>) -> Result<VoteBuilder<C>> {
        let heaviest_tip = votes
            .heaviest_element()
            .cloned()
            .expect("votes must not be empty");

        Ok(VoteBuilder {
            issuer: Issuer::Genesis,
            issuing_time: heaviest_tip.issuing_time,
            committee: heaviest_tip.committee.clone(),
            config: heaviest_tip.config.clone(),
            consensus: heaviest_tip.consensus.clone(),
            cumulative_slot_weight: heaviest_tip.cumulative_slot_weight,
            round: heaviest_tip.round,
            leader_weight: heaviest_tip.leader_weight,
            votes_by_issuer: VotesByIssuer::try_from(votes)?.into(),
        })
    }

    pub fn build(mut self, issuer: Id<C::IssuerID>, issuing_time: u64) -> Result<Vote<C>> {
        self.issuer = Issuer::User(issuer.clone());
        if let Some(new_slot) = self.update_issuing_time(issuing_time) {
            self.remove_offline_committee_members(new_slot)?;
        }

        if let Some(committee_member) = self.committee.member(&issuer).cloned() {
            self.committee = self.committee.set_online(committee_member.key(), true);

            let (consensus_threshold, confirm) = self.determine_consensus_threshold();
            if let Some(seen_weights) = self.can_vote(committee_member.key(), consensus_threshold)? {
                self.consensus = ConsensusMechanism::run(&self, consensus_threshold)?.into();

                // advance the round if the acceptance threshold is now met
                if seen_weights + committee_member.weight() >= consensus_threshold {
                    self.leader_weight = self.config.leader_weight(&self);
                    self.round += 1;
                }

                return Ok(Vote::from(Arc::new_cyclic(|me| {
                    self.votes_by_issuer
                        .insert(committee_member.key().clone(), VoteRefs::from_iter([me.into()]));
                    self
                })))
            }
        }

        Ok(Vote::from(Arc::new(self)))
    }

    pub fn build_genesis(config: C) -> Self {
        Self {
            issuer: Issuer::Genesis,
            issuing_time: config.genesis_time(),
            committee: config.select_committee(None),
            config: Arc::new(config),
            consensus: ConsensusCommitment::default(),
            cumulative_slot_weight: 0,
            round: 0,
            leader_weight: 0,
            votes_by_issuer: VoteRefsByIssuer::default(),
        }
    }

    fn determine_consensus_threshold(&self) -> (u64, bool) {
        if self.committee.online_weight() >= self.committee.confirmation_threshold() {
            (self.committee.confirmation_threshold(), true)
        } else {
            (self.committee.acceptance_threshold(), false)
        }
    }

    fn can_vote(&self, member_id: &Id<C::IssuerID>, threshold: u64) -> Result<Option<u64>> {
        let seen_weight = self.referenced_round_weight()?;
        if let Some(own_votes) = self.votes_by_issuer.get(member_id) {
            if let Some(own_vote) = own_votes.iter().next() {
                if Vote::try_from(own_vote)?.round == self.round && seen_weight < threshold {
                    return Ok(None);
                }
            }
        }

        Ok(Some(seen_weight))
    }

    fn referenced_round_weight(&self) -> Result<u64> {
        let mut latest_round = 0;
        let mut referenced_round_weight = 0;

        for (issuer, votes) in &self.votes_by_issuer {
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

    fn update_issuing_time(&mut self, issuing_time: u64) -> Option<u64> {
        let old_slot = self.config.slot_oracle(&self);
        self.issuing_time = issuing_time;
        let new_slot = self.config.slot_oracle(&self);

        (new_slot > old_slot).then_some(new_slot)
    }

    fn remove_offline_committee_members(&mut self, slot: u64) -> Result<()> {
        for member in self.committee_members_idle_since(slot - self.config.offline_threshold())? {
            self.committee = self.committee.set_online(&member, false);
        }
        Ok(())
    }

    fn committee_members_idle_since(&self, slot: u64) -> Result<HashSet<Id<C::IssuerID>>> {
        let mut idle_members = HashSet::new();
        for member in self.committee.iter() {
            if member.is_online() && self.committee_member_idle_since(&member, slot)? {
                idle_members.insert(member.key().clone());
            }
        }
        Ok(idle_members)
    }

    fn committee_member_idle_since(&self, member: &Member<C::IssuerID>, slot: u64) -> Result<bool> {
        let mut is_idle = true;
        if let Some(votes) = self.votes_by_issuer.get(member.key()) {
            for vote in votes.iter() {
                if self.config.slot_oracle(&(*Vote::try_from(vote)?)) >= slot {
                    is_idle = false;
                }
            }
        };

        Ok(is_idle)
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
