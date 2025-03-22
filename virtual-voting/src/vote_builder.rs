use std::{cmp::max, collections::HashSet, sync::Arc};

use types::{
    bft::{Committee, Member},
    ids::{BlockID, IssuerID},
};

use crate::{
    Config,
    Error::{TimeMustIncrease, VotesMustNotBeEmpty},
    Issuer, Milestone, Result, VirtualVoting, Vote, VoteRef, VoteRefs, VoteRefsByIssuer, Votes,
    VotesByIssuer,
};

pub struct VoteBuilder<T: Config> {
    pub block_id: BlockID,
    pub config: Arc<T>,
    pub issuer: Issuer,
    pub time: u64,
    pub slot: u64,
    pub cumulative_slot_weight: u64,
    pub round: u64,
    pub referenced_round_weight: u64,
    pub committee: Committee,
    pub referenced_milestones: VoteRefsByIssuer<T>,
    pub milestone: Option<Milestone<T>>,
}

impl<C: Config> VoteBuilder<C> {
    pub(crate) fn build(
        block_id: BlockID,
        issuer: &IssuerID,
        time: u64,
        votes: &Votes<C>,
    ) -> Result<Vote<C>> {
        // determine heaviest vote
        let Some(heaviest_vote) = votes.heaviest_element() else {
            return Err(VotesMustNotBeEmpty);
        };

        // copy perception of heaviest vote
        let mut builder = VoteBuilder {
            block_id,
            issuer: Issuer::User(issuer.clone()),
            time,
            slot: heaviest_vote.slot,
            committee: heaviest_vote.committee.clone(),
            cumulative_slot_weight: heaviest_vote.cumulative_slot_weight,
            config: heaviest_vote.config.clone(),
            round: heaviest_vote.round,
            referenced_round_weight: 0,
            referenced_milestones: VoteRefsByIssuer::default(),
            milestone: None,
        };

        // aggregate information from remaining votes
        let (referenced_milestones, latest_vote) = builder.aggregate_votes(votes);

        // check if time is monotonically increasing
        if builder.time < latest_vote.time {
            return Err(TimeMustIncrease);
        }

        // update validator availability on slot transition
        if builder.slot > latest_vote.slot {
            for member in builder.offline_validators(&referenced_milestones) {
                builder.committee = builder.committee.set_online(&member, false);
            }
        }

        // TODO: ROTATE COMMITTEE

        // build validator perception if issuer is part of the committee
        if let Some(validator) = builder.committee.member(issuer).cloned() {
            return builder.build_validator_perception(&validator, referenced_milestones);
        }

        // wrap final state in a Vote and return
        Ok(Vote::from(Arc::new(builder)))
    }

    pub(crate) fn build_genesis(config: C) -> Vote<C> {
        Vote::from(Arc::new_cyclic(|me| {
            let committee = config.select_committee(None);

            Self {
                block_id: BlockID::default(),
                issuer: Issuer::Genesis,
                time: config.genesis_time(),
                slot: 0,
                committee: committee.clone(),
                config: Arc::new(config),
                cumulative_slot_weight: 0,
                round: 0,
                referenced_round_weight: u64::MAX,
                referenced_milestones: VoteRefsByIssuer::from_committee(
                    &committee,
                    &VoteRef::from(me),
                ),
                milestone: Some(Milestone {
                    height: 0,
                    leader_weight: u64::MAX,
                    accepted: me.into(),
                    confirmed: me.into(),
                    prev: me.into(),
                    slot_boundary: me.into(),
                }),
            }
        }))
    }

    fn build_validator_perception(
        mut self,
        validator: &Member,
        votes: VotesByIssuer<C>,
    ) -> Result<Vote<C>> {
        // set ourselves online before voting to also consider our own weight
        self.committee = self.committee.set_online(validator.key(), true);

        // determine consensus threshold (switch between confirmation and acceptance)
        let (threshold, does_confirm) = self.consensus_threshold();

        // check if we can commit (haven't voted yet for this round or have enough weight)
        if votes.get(validator.key()).is_none_or(|validator_votes| {
            validator_votes.round() < self.round || self.referenced_round_weight >= threshold
        }) {
            // determine the heaviest tip and the accepted milestone
            let (accepted, heaviest_tip) = VirtualVoting::run(votes, &self.committee, threshold)?;

            // update the round and referenced round weight if we have enough weight
            if self.referenced_round_weight + validator.weight() >= threshold {
                self.round += 1;
                self.referenced_round_weight = validator.weight();
            }

            // build milestone
            self.milestone = Some(Milestone {
                height: heaviest_tip.milestone()?.height + 1,
                leader_weight: self.config.leader_weight(&self),
                prev: (&heaviest_tip).into(),
                accepted: (&accepted).into(),
                confirmed: match does_confirm {
                    // if we have enough weight to confirm, use the accepted milestone
                    true => (&accepted).into(),
                    // otherwise inherit the heaviest tip's confirmed milestone
                    false => heaviest_tip.confirmed_milestone()?.clone(),
                },
                slot_boundary: match self.slot > heaviest_tip.slot {
                    // if we are in a new slot, use the heaviest tip as slot boundary
                    true => (&heaviest_tip).into(),
                    // otherwise inherit the heaviest tip's slot boundary
                    false => heaviest_tip.slot_boundary()?.clone(),
                },
            });

            // update cumulative slot weight
            let prev_accepted_slot = heaviest_tip.accepted_milestone()?.slot;
            self.cumulative_slot_weight += accepted.slot_weight_since(prev_accepted_slot)?;

            // build milestone vote and insert into referenced milestones
            return Ok(Vote::from(Arc::new_cyclic(|me| {
                self.referenced_milestones
                    .insert(validator.key().clone(), VoteRefs::from_iter([me.into()]));
                self
            })));
        }

        // wrap final state in a Vote and return
        Ok(Vote::from(Arc::new(self)))
    }

    fn aggregate_votes(&mut self, votes: &Votes<C>) -> (VotesByIssuer<C>, Vote<C>) {
        let mut referenced_milestones = VotesByIssuer::default();
        let mut seen_voters = HashSet::new();
        let mut latest_vote = (0, None);

        for vote in votes {
            latest_vote = max(latest_vote, (vote.time, Some(vote)));

            for (issuer, milestone_refs) in &vote.referenced_milestones {
                if let Some(committee_member) = self.committee.member(issuer) {
                    let mut milestones = Votes::default();

                    for milestone in milestone_refs.iter().filter_map(|m| Vote::try_from(m).ok()) {
                        if milestone.round == self.round && seen_voters.insert(issuer.clone()) {
                            self.referenced_round_weight += committee_member.weight();
                        }
                        milestones.insert(milestone);
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

    fn offline_validators(&self, votes: &VotesByIssuer<C>) -> HashSet<IssuerID> {
        let offline_threshold = self.slot - self.config.offline_threshold();

        // filter online validators that haven't voted since the offline threshold
        self.committee
            .iter()
            .filter(|member| {
                member.is_online()
                    && !votes
                        .get(member.key())
                        .is_some_and(|m| m.into_iter().any(|m| m.slot >= offline_threshold))
            })
            .map(|member| member.key().clone())
            .collect()
    }

    fn consensus_threshold(&self) -> (u64, bool) {
        // calculate acceptance and confirmation thresholds
        let online_weight = self.committee.online_weight();
        let total_weight = self.committee.total_weight();
        let acceptance_threshold = online_weight - online_weight / 3;
        let confirmation_threshold = total_weight - total_weight / 3;

        // ebb and flow between acceptance and confirmation thresholds
        match self.committee.online_weight() >= confirmation_threshold {
            // if we have enough online weight to confirm, use confirmation threshold
            true => (confirmation_threshold, true),
            // otherwise use acceptance threshold
            false => (acceptance_threshold, false),
        }
    }
}
