use std::{
    cmp::{Ordering, max},
    collections::HashMap,
};

use crate::{
    Committee, ConfigInterface, Vote, Votes, VotesByIssuer, VotesByRound,
    consensus::WalkResult::{LatestAcceptedMilestoneFound, PreviousRoundTargets},
    errors::Error,
};

pub(crate) struct ConsensusRound<ID: ConfigInterface> {
    committee: Committee<ID>,
    children: HashMap<Vote<ID>, Votes<ID>>,
    weights: HashMap<Vote<ID>, u64>,
}

impl<ID: ConfigInterface> ConsensusRound<ID> {
    pub(crate) fn new(committee: Committee<ID>) -> Self {
        Self {
            committee,
            children: HashMap::new(),
            weights: HashMap::new(),
        }
    }

    /// Walks through the votes of each round and returns the latest accepted
    /// milestone.
    pub(crate) fn latest_accepted_milestone(
        &mut self,
        mut votes_by_round: VotesByRound<ID>,
    ) -> Result<Vote<ID>, Error> {
        for round in (0..=votes_by_round.max_round()).rev() {
            match self.heaviest_target(votes_by_round.fetch(round))? {
                LatestAcceptedMilestoneFound(latest_accepted_milestone) => {
                    return Ok(latest_accepted_milestone);
                }
                PreviousRoundTargets(previous_round_targets) => {
                    if round > 0 {
                        votes_by_round.insert_votes_by_issuer(round - 1, previous_round_targets)
                    }
                }
            }
        }

        unreachable!("we should never reach this point in the logic as the root is always accepted")
    }

    pub(crate) fn heaviest_descendant(&self, vote: &Vote<ID>) -> Vote<ID> {
        let mut heaviest_descendant = vote.clone();
        while let Some(children) = self.children.get(&heaviest_descendant) {
            match children.heaviest(&self.weights) {
                Some(heaviest_child) => heaviest_descendant = heaviest_child,
                None => break,
            }
        }

        heaviest_descendant
    }

    fn add_weight(&mut self, vote: &Vote<ID>, weight: u64) -> u64 {
        *self
            .weights
            .entry(vote.clone())
            .and_modify(|w| *w += weight)
            .or_insert(weight)
    }

    fn heaviest_target(
        &mut self,
        votes_of_round: &VotesByIssuer<ID>,
    ) -> Result<WalkResult<ID>, Error> {
        let mut targets = VotesByIssuer::default();
        let mut heaviest_vote = None;
        let mut heaviest_weight = 0;

        for (issuer, votes) in votes_of_round.iter() {
            for vote in votes.iter() {
                let target = vote.target().try_into()?;

                if vote.is_accepted() {
                    return Ok(LatestAcceptedMilestoneFound(vote.clone()));
                }

                let updated_weight =
                    self.add_weight(&target, self.committee.member_weight(vote.issuer()));
                match updated_weight.cmp(&heaviest_weight) {
                    Ordering::Greater => {
                        heaviest_vote = Some(target.clone());
                        heaviest_weight = updated_weight;
                    }
                    Ordering::Equal => {
                        heaviest_vote = max(heaviest_vote, Some(target.clone()));
                    }
                    Ordering::Less => continue,
                }

                targets.fetch(issuer).insert(target.clone());
                self.children
                    .entry(target)
                    .or_default()
                    .insert(vote.clone());
            }
        }

        match heaviest_weight > (2.0 / 3.0 * self.committee.online_weight() as f64) as u64 {
            true => Ok(LatestAcceptedMilestoneFound(heaviest_vote.unwrap())),
            false => Ok(PreviousRoundTargets(targets)),
        }
    }
}

enum WalkResult<ID: ConfigInterface> {
    LatestAcceptedMilestoneFound(Vote<ID>),
    PreviousRoundTargets(VotesByIssuer<ID>),
}
