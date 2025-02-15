use std::{cmp::max, collections::HashSet, sync::Arc};

use committee::{Committee, Member};
use utils::Id;

use crate::{
    Config,
    Error::{TimeMustIncrease, VotesMustNotBeEmpty},
    Issuer,
    Issuer::User,
    Milestone, Result, VirtualVoting, Vote, VoteRef, VoteRefs, VoteRefsByIssuer, Votes,
    VotesByIssuer, VotesByRound,
};

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
    pub(crate) fn build_genesis(config: C) -> Vote<C> {
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
                referenced_milestones: VoteRefsByIssuer::from_committee(
                    &committee,
                    &VoteRef::from(me),
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

    pub(crate) fn build(issuer: &Id<C::IssuerID>, time: u64, votes: &Votes<C>) -> Result<Vote<C>> {
        let Some(heaviest_vote) = votes.heaviest_element() else {
            return Err(VotesMustNotBeEmpty);
        };

        let mut builder = heaviest_vote.copy_perception(issuer, time);
        let (referenced_milestones, latest_vote) = builder.aggregate_knowledge(votes);

        if builder.time < latest_vote.time {
            return Err(TimeMustIncrease);
        } else if builder.slot > latest_vote.slot {
            builder.flag_offline_validators(&referenced_milestones)?;
        }

        // TODO: UPDATE COMMITTEE

        if let Some(validator) = builder.committee.member(issuer).cloned() {
            return builder.build_validator_perception(&validator, referenced_milestones);
        }

        Ok(Vote::from(Arc::new(builder)))
    }

    fn copy_perception(&self, issuer: &Id<C::IssuerID>, time: u64) -> VoteBuilder<C> {
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

        let mut seen_voters = HashSet::new();
        for vote in votes {
            latest_vote = max(latest_vote, (vote.time, Some(vote)));

            for (issuer, milestone_refs) in &vote.referenced_milestones {
                if let Some(committee_member) = self.committee.member(issuer) {
                    let mut milestones = Votes::default();
                    for milestone_ref in milestone_refs {
                        if let Ok(milestone) = Vote::try_from(milestone_ref) {
                            if milestone.round == self.round && seen_voters.insert(issuer.clone()) {
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

    fn build_validator_perception(
        mut self,
        validator: &Member<C::IssuerID>,
        votes: VotesByIssuer<C>,
    ) -> Result<Vote<C>> {
        // set ourselves online before voting to also consider our own weight
        self.committee = self.committee.set_online(validator.key(), true);

        // determine consensus threshold (switch between confirmation and acceptance)
        let (threshold, does_confirm) = self.consensus_threshold();

        // check if we should commit (haven't voted yet for this round or have enough weight)
        if self.should_commit(validator.key(), threshold, &votes) {
            // determine the heaviest tip and the accepted milestone
            let (accepted, heaviest_tip) = VirtualVoting::run(votes, &self.committee, threshold)?;

            // update the round and referenced round weight if we have enough weight
            if self.referenced_round_weight + validator.weight() >= threshold {
                self.round += 1;
                self.referenced_round_weight = validator.weight();
            }

            // update the milestone
            self.milestone = Some(Milestone {
                leader_weight: self.config.leader_weight(&self),
                prev: (&heaviest_tip).into(),
                accepted: (&accepted).into(),
                confirmed: match does_confirm {
                    true => (&accepted).into(),
                    false => heaviest_tip.confirmed_milestone()?.clone(),
                },
            });

            // finally update the slot weight
            let prev_accepted = Vote::try_from(heaviest_tip.accepted_milestone()?)?;
            self.update_slot_weight(accepted, prev_accepted)?;

            // build milestone vote and insert it into the referenced milestones
            return Ok(Vote::from(Arc::new_cyclic(|me| {
                self.referenced_milestones
                    .insert(validator.key().clone(), VoteRefs::from_iter([me.into()]));
                self
            })));
        }

        Ok(Vote::from(Arc::new(self)))
    }

    fn consensus_threshold(&self) -> (u64, bool) {
        if self.committee.online_weight() >= self.committee.confirmation_threshold() {
            (self.committee.confirmation_threshold(), true)
        } else {
            (self.committee.acceptance_threshold(), false)
        }
    }

    fn should_commit(&self, id: &Id<C::IssuerID>, threshold: u64, votes: &VotesByIssuer<C>) -> bool {
        votes
            .get(id)
            .and_then(|m| m.heaviest_element())
            .map_or(true, |v| {
                v.round != self.round || self.referenced_round_weight >= threshold
            })
    }

    fn update_slot_weight(&mut self, mut accepted: Vote<C>, prev_accepted: Vote<C>) -> Result<()> {
        while accepted.slot != prev_accepted.slot {
            accepted = accepted.find_slot_boundary()?;

            self.slot_weight += accepted.committee.online_weight();
        }
        Ok(())
    }
}
