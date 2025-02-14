use std::{cmp::max, collections::HashSet, sync::Arc};

use committee::{Committee, Member};
use utils::Id;

use crate::{
    Config, Error,
    Error::{NoMilestone, VotesMustNotBeEmpty},
    Issuer,
    Issuer::User,
    Milestone, Result, VirtualVoting, Vote, VoteRefs, VoteRefsByIssuer, Votes, VotesByIssuer,
};
use crate::Error::TimeMustIncrease;

pub struct VoteBuilder<T: Config> {
    pub config: Arc<T>,
    pub issuer: Issuer<T::IssuerID>,
    pub time: u64,
    pub slot: u64,
    pub slot_weight: u64,
    pub round: u64,
    pub referenced_round_weight: u64,
    pub committee: Committee<T::IssuerID>,
    pub referenced_milestones: VoteRefsByIssuer<T>,
    pub milestone: Option<Milestone<T>>,
}

impl<C: Config> VoteBuilder<C> {
    pub fn build_genesis(config: C) -> Vote<C> {
        Vote::from(Arc::new_cyclic(|me| {
            let committee = config.select_committee(None);

            Self {
                issuer: Issuer::Genesis,
                time: config.genesis_time(),
                slot: 0,
                committee: committee.clone(),
                config: Arc::new(config),
                slot_weight: 0,
                round: 0,
                referenced_round_weight: u64::MAX,
                referenced_milestones: VoteRefsByIssuer::from_iter(
                    committee
                        .iter()
                        .map(|member| (member.key().clone(), VoteRefs::from_iter([me.into()]))),
                ),
                milestone: Some(Milestone {
                    leader_weight: u64::MAX,
                    accepted: me.into(),
                    confirmed: me.into(),
                    prev: me.into(),
                }),
            }
        }))
    }

    pub fn build(issuer: &Id<C::IssuerID>, time: u64, votes: &Votes<C>) -> Result<Vote<C>> {
        let Some(heaviest_vote) = votes.heaviest_element() else {
            return Err(VotesMustNotBeEmpty);
        };

        let mut builder = heaviest_vote.inherit_perception(issuer, time);
        let (referenced_milestones, latest_vote) = builder.aggregate_knowledge(votes);

        if builder.time < latest_vote.time {
            return Err(TimeMustIncrease);
        }
        if builder.slot > latest_vote.slot {
            builder.flag_offline_validators(&referenced_milestones)?;
        }

        // TODO: UPDATE COMMITTEE

        if let Some(validator) = builder.committee.member(issuer).cloned() {
            return builder.build_validator_vote(&validator, referenced_milestones);
        }

        Ok(Vote::from(Arc::new(builder)))
    }

    fn inherit_perception(&self, issuer: &Id<C::IssuerID>, time: u64) -> VoteBuilder<C> {
        VoteBuilder {
            issuer: User(issuer.clone()),
            time,
            slot: self.config.slot_oracle(time),
            committee: self.committee.clone(),
            slot_weight: self.slot_weight,
            config: self.config.clone(),
            round: self.round,
            referenced_round_weight: 0,
            referenced_milestones: VoteRefsByIssuer::default(),
            milestone: None,
        }
    }

    fn aggregate_knowledge(&mut self, votes: &Votes<C>) -> (VotesByIssuer<C>, Vote<C>) {
        let mut referenced_milestones = VotesByIssuer::default();
        let mut latest_vote = (0, None);

        let mut seen_issuers = HashSet::new();
        for vote in votes {
            latest_vote = max(latest_vote, (vote.time, Some(vote)));
            self.time = max(self.time, vote.time);

            for (issuer, milestone_refs) in &vote.referenced_milestones {
                if let Some(committee_member) = self.committee.member(issuer) {
                    let mut milestones = Votes::default();
                    for milestone_ref in milestone_refs {
                        if let Ok(milestone) = Vote::try_from(milestone_ref) {
                            if milestone.round == self.round && seen_issuers.insert(issuer.clone())
                            {
                                self.referenced_round_weight += committee_member.weight();
                            }
                            milestones.insert(milestone);
                        }
                    }

                    referenced_milestones.insert_or_update((issuer.clone(), milestones));
                }
            }
        }

        self.referenced_milestones = (&referenced_milestones).into();

        (
            referenced_milestones,
            latest_vote.1.expect("must exist").clone(),
        )
    }

    fn build_validator_vote(
        mut self,
        validator: &Member<C::IssuerID>,
        referenced_milestones: VotesByIssuer<C>,
    ) -> Result<Vote<C>> {
        // set ourselves online before voting to also consider our weight
        self.committee = self.committee.set_online(validator.key(), true);

        // determine consensus threshold (switch between confirmation and acceptance)
        let (threshold, does_confirm) = self.consensus_threshold();

        // check if we should vote
        if self.should_vote(validator.key(), threshold, &referenced_milestones) {
            // run virtual voting algorithm
            let (accepted, prev) = VirtualVoting::run(&self, threshold)?;

            // update consensus weights
            if accepted.slot > Vote::try_from(&accepted.milestone()?.prev)?.slot {
                self.slot_weight += prev.committee.online_weight();
            }
            if self.referenced_round_weight + validator.weight() >= threshold {
                self.round += 1;
            }

            self.milestone = Some(Milestone {
                leader_weight: self.config.leader_weight(&self),
                prev: (&prev).into(),
                accepted: (&accepted).into(),
                confirmed: match does_confirm {
                    true => (&accepted).into(),
                    false => prev
                        .milestone
                        .as_ref()
                        .ok_or(NoMilestone)?
                        .confirmed
                        .clone(),
                },
            });

            // build vote and update votes_by_issuer map
            return Ok(Vote::from(Arc::new_cyclic(|me| {
                self.referenced_milestones
                    .insert(validator.key().clone(), VoteRefs::from_iter([me.into()]));
                self
            })));
        }

        Ok(Vote::from(Arc::new(self)))
    }

    pub fn milestone(&self) -> Result<&Milestone<C>> {
        self.milestone.as_ref().ok_or(Error::NoMilestone)
    }

    fn consensus_threshold(&self) -> (u64, bool) {
        if self.committee.online_weight() >= self.committee.confirmation_threshold() {
            (self.committee.confirmation_threshold(), true)
        } else {
            (self.committee.acceptance_threshold(), false)
        }
    }

    fn should_vote(
        &self,
        issuer: &Id<C::IssuerID>,
        consensus_threshold: u64,
        referenced_milestones: &VotesByIssuer<C>,
    ) -> bool {
        referenced_milestones
            .get(issuer)
            .and_then(|m| m.heaviest_element())
            .map_or(true, |issuer_vote| {
                issuer_vote.round != self.round
                    || self.referenced_round_weight >= consensus_threshold
            })
    }

    fn flag_offline_validators(&mut self, referenced_milestones: &VotesByIssuer<C>) -> Result<()> {
        for member in self.validators_offline_since(
            self.slot - self.config.offline_threshold(),
            referenced_milestones,
        )? {
            self.committee = self.committee.set_online(&member, false);
        }
        Ok(())
    }

    fn validators_offline_since(
        &self,
        slot: u64,
        referenced_milestones: &VotesByIssuer<C>,
    ) -> Result<HashSet<Id<C::IssuerID>>> {
        let mut offline_validators = HashSet::new();
        for member in self.committee.iter() {
            if member.is_online()
                && self.validator_offline_since(member.key(), slot, referenced_milestones)?
            {
                offline_validators.insert(member.key().clone());
            }
        }
        Ok(offline_validators)
    }

    fn validator_offline_since(
        &self,
        id: &Id<C::IssuerID>,
        slot: u64,
        referenced_milestones: &VotesByIssuer<C>,
    ) -> Result<bool> {
        let mut is_offline = true;
        if let Some(validator_milestones) = referenced_milestones.get(id) {
            for validator_milestone in validator_milestones {
                if validator_milestone.slot >= slot {
                    is_offline = false;
                }
            }
        };

        Ok(is_offline)
    }
}
