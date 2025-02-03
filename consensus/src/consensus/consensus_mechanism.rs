use std::{
    cmp::{Ordering, max},
    collections::HashMap,
};

use crate::{Committee, ConfigInterface, Result, Vote, Votes, VotesByIssuer, VotesByRound};

pub(crate) struct ConsensusMechanism<ID: ConfigInterface> {
    committee: Committee<ID>,
    children: HashMap<Vote<ID>, Votes<ID>>,
    weights: HashMap<Vote<ID>, u64>,
    pub(crate) last_accepted_milestone: Option<Vote<ID>>,
    pub(crate) last_confirmed_milestone: Option<Vote<ID>>,
    pub(crate) heaviest_tip: Option<Vote<ID>>,
}

impl<C: ConfigInterface> ConsensusMechanism<C> {
    pub(crate) fn new(committee: Committee<C>) -> Self {
        Self {
            committee,
            children: HashMap::new(),
            weights: HashMap::new(),
            last_accepted_milestone: None,
            last_confirmed_milestone: None,
            heaviest_tip: None,
        }
    }

    /// Walks through the votes of each round and returns the latest accepted
    /// milestone.
    pub(crate) fn scan_past_cone(&mut self, mut votes_by_round: VotesByRound<C>) -> Result<()> {
        for round in (0..=votes_by_round.max_round()).rev() {
            let previous_round_targets = self.heaviest_target(votes_by_round.fetch(round))?;
            if !previous_round_targets.is_empty() {
                votes_by_round.extend(round - 1, previous_round_targets);
            } else {
                break;
            }
        }

        Ok(())
    }

    pub(crate) fn scan_future_cone(&mut self) {
        let Some(mut heaviest_tip) = self.last_accepted_milestone.clone() else {
            return;
        };
        while let Some(children) = self.children.get(&heaviest_tip) {
            match self.heaviest_child(children) {
                Some(heaviest_child) => heaviest_tip = heaviest_child,
                None => break,
            }
        }

        self.heaviest_tip = Some(heaviest_tip);
    }

    fn add_weight(&mut self, vote: &Vote<C>, weight: u64) -> u64 {
        *self
            .weights
            .entry(vote.clone())
            .and_modify(|w| *w += weight)
            .or_insert(weight)
    }

    fn heaviest_target(&mut self, votes_of_round: &VotesByIssuer<C>) -> Result<VotesByIssuer<C>> {
        let mut targets = VotesByIssuer::default();
        let mut heaviest_vote = None;
        let mut heaviest_weight = 0;

        for (issuer, votes) in votes_of_round {
            for vote in votes {
                let target = Vote::try_from(&vote.consensus_view.heaviest_tip)?;
                let updated_weight = self.add_weight(&target, self.committee.member_weight(issuer));
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

                if !vote.consensus_view.heaviest_tip.points_to(vote) {
                    targets.fetch(issuer.clone()).insert(target.clone());

                    self.children
                        .entry(target.clone())
                        .or_default()
                        .insert(vote.clone());
                }
            }
        }

        println!("heaviest_vote: {:?} ({:?})", heaviest_vote, heaviest_weight);

        if self.last_accepted_milestone.is_none()
            && heaviest_weight >= self.committee.acceptance_threshold()
        {
            self.last_accepted_milestone = heaviest_vote.clone();
        }

        if self.last_confirmed_milestone.is_none()
            && heaviest_weight >= self.committee.confirmation_threshold()
        {
            self.last_confirmed_milestone = heaviest_vote.clone();

            return Ok(VotesByIssuer::default());
        }

        Ok(targets)
    }

    fn heaviest_child(&self, votes: &Votes<C>) -> Option<Vote<C>> {
        votes
            .into_iter()
            .map(|candidate_weak| {
                (
                    candidate_weak.clone(),
                    self.weights.get(candidate_weak).unwrap_or(&0),
                )
            })
            .max_by(|(candidate1, weight1), (candidate2, weight2)| {
                weight1
                    .cmp(weight2)
                    .then_with(|| candidate1.cmp(candidate2))
            })
            .map(|(candidate, _)| candidate)
    }
}
