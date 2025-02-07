use std::{cmp::max, collections::HashMap};
use committee::Committee;
use crate::{
    ConfigInterface, ConsensusCommitment, Error, Result, Vote, VoteBuilder, Votes,
    VotesByIssuer, VotesByRound, consensus::vote_tracker::VoteTracker,
};

pub struct ConsensusMechanism<C: ConfigInterface> {
    children: HashMap<Vote<C>, Votes<C>>,
    vote_tracker: VoteTracker<C>,
    consensus_threshold: u64,
}

impl<C: ConfigInterface> ConsensusMechanism<C> {
    pub fn run(vote: &VoteBuilder<C>, consensus_threshold: u64) -> Result<ConsensusCommitment<C>> {
        let votes_by_round = VotesByIssuer::try_from(&vote.votes_by_issuer)?.into();

        let mut consensus_mechanism = Self::new(vote.committee.clone(), consensus_threshold);
        let milestone = consensus_mechanism.milestone(votes_by_round)?;
        let heaviest_tip = consensus_mechanism.find_heaviest_tip(&milestone);

        Ok(ConsensusCommitment {
            milestone: milestone.into(),
            tip: heaviest_tip.into(),
        })
    }

    fn new(committee: Committee<C::IssuerID>, consensus_threshold: u64) -> Self {
        Self {
            children: HashMap::new(),
            vote_tracker: VoteTracker::new(committee.clone()),
            consensus_threshold,
        }
    }

    fn milestone(&mut self, mut rounds: VotesByRound<C>) -> Result<Vote<C>> {
        for round in (0..=rounds.max_round()).rev() {
            let mut next_votes = VotesByIssuer::default();
            let mut heaviest = (0, None);

            for (issuer, issuer_votes) in rounds.fetch(round) {
                for vote in &*issuer_votes {
                    heaviest = max(heaviest, self.vote_tracker.track_vote(vote, issuer));

                    if !vote.consensus.tip.points_to(vote) {
                        let target = Vote::try_from(&vote.consensus.tip)?;

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

    fn find_heaviest_tip(&mut self, milestone: &Vote<C>) -> Vote<C> {
        let mut heaviest_tip = milestone.clone();
        while let Some(heaviest_child) = self
            .children
            .get(&heaviest_tip)
            .and_then(|c| self.vote_tracker.heaviest_vote(c))
        {
            heaviest_tip = heaviest_child;
        }

        heaviest_tip
    }
}
