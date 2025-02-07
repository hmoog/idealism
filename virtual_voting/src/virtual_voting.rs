use std::{cmp::max, collections::HashMap};

use crate::{
    ConfigInterface, Error, Result, Vote, VoteBuilder, Votes, VotesByIssuer, VotesByRound,
    WeightTracker,
};

pub struct VirtualVoting<C: ConfigInterface> {
    children: HashMap<Vote<C>, Votes<C>>,
    weight_tracker: WeightTracker<C>,
    consensus_threshold: u64,
}

impl<C: ConfigInterface> VirtualVoting<C> {
    pub fn run(vote: &VoteBuilder<C>, consensus_threshold: u64) -> Result<(Vote<C>, Vote<C>)> {
        let votes_by_round = VotesByRound::from(VotesByIssuer::try_from(&vote.votes_by_issuer)?);

        let mut virtual_voting = Self {
            children: HashMap::new(),
            weight_tracker: WeightTracker::new(vote.committee.clone()),
            consensus_threshold,
        };

        let milestone = virtual_voting.milestone(votes_by_round)?;
        let heaviest_tip = virtual_voting.heaviest_tip(&milestone);

        Ok((milestone, heaviest_tip))
    }

    fn milestone(&mut self, mut rounds: VotesByRound<C>) -> Result<Vote<C>> {
        for round in (0..=rounds.max_round()).rev() {
            let mut next_votes = VotesByIssuer::default();
            let mut heaviest = (0, None);

            for (issuer, issuer_votes) in rounds.fetch(round) {
                for vote in &*issuer_votes {
                    heaviest = max(heaviest, self.weight_tracker.weight_entry(vote, issuer));

                    if !vote.heaviest_tip.points_to(vote) {
                        let target = Vote::try_from(&vote.heaviest_tip)?;

                        self.children
                            .entry(target.clone())
                            .or_default()
                            .insert(vote.clone());
                        next_votes.fetch(issuer.clone()).insert(target);
                    }
                }
            }

            if heaviest.0 >= self.consensus_threshold {
                return Ok(heaviest.1.expect("heaviest vote should be set"));
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
