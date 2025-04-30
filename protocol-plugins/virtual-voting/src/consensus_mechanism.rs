use std::{cmp::max, collections::HashMap};

use common::bft::Committee;

use crate::{
    Error, Result, VirtualVotingConfig, Vote, Votes, VotesByIssuer, VotesByRound, WeightTracker,
};

pub struct ConsensusMechanism<C: VirtualVotingConfig> {
    children: HashMap<Vote<C>, Votes<C>>,
    weight_tracker: WeightTracker<C>,
    consensus_threshold: u64,
}

impl<C: VirtualVotingConfig> ConsensusMechanism<C> {
    pub fn run(
        votes: VotesByIssuer<C>,
        committee: &Committee,
        threshold: u64,
    ) -> Result<(Vote<C>, Vote<C>)> {
        let mut virtual_voting = Self {
            children: HashMap::new(),
            weight_tracker: WeightTracker::new(committee.clone()),
            consensus_threshold: threshold,
        };

        let accepted = virtual_voting.accepted(VotesByRound::from(votes))?;
        let heaviest_tip = virtual_voting.heaviest_tip(&accepted);

        Ok((accepted, heaviest_tip))
    }

    fn accepted(&mut self, mut rounds: VotesByRound<C>) -> Result<Vote<C>> {
        for round in (0..=rounds.max_round()).rev() {
            let mut next_votes = VotesByIssuer::default();
            let mut heaviest = (0, None);

            for (issuer, issuer_votes) in rounds.fetch(round) {
                for vote in &*issuer_votes {
                    heaviest = max(heaviest, self.weight_tracker.weight_entry(vote, issuer));

                    if !vote.milestone()?.prev.points_to(vote) {
                        let target = Vote::try_from(&vote.milestone()?.prev)?;

                        self.children
                            .entry(target.clone())
                            .or_default()
                            .insert(vote.clone());
                        next_votes.fetch(issuer.clone()).insert(target);
                    }
                }
            }

            if heaviest.0 >= self.consensus_threshold {
                return Ok(heaviest.1.expect("must exist"));
            } else if round == 0 || next_votes.is_empty() {
                break;
            }

            rounds.extend(round - 1, next_votes);
        }

        Err(Error::NoConfirmedMilestoneInPastCone)
    }

    fn heaviest_tip(&mut self, milestone: &Vote<C>) -> Vote<C> {
        let mut heaviest_tip = milestone.clone();
        while let Some(heaviest_child) = self
            .children
            .get(&heaviest_tip)
            .and_then(|c| self.weight_tracker.heaviest_vote(c))
        {
            heaviest_tip = heaviest_child;
        }

        heaviest_tip
    }
}
