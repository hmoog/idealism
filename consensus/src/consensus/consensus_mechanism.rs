use std::{cmp::max, collections::HashMap};
use committee::Committee;
use crate::{
    ConfigInterface, ConsensusCommitment, Error, Result, Vote, VoteBuilder, Votes,
    VotesByIssuer, VotesByRound, consensus::vote_tracker::VoteTracker,
};

pub struct ConsensusMechanism<ID: ConfigInterface> {
    committee: Committee<ID::IssuerID>,
    children: HashMap<Vote<ID>, Votes<ID>>,
    vote_tracker: VoteTracker<ID>,
    accepted: Option<Vote<ID>>,
    confirmed: Option<Vote<ID>>,
    heaviest_tip: Option<Vote<ID>>,
}

impl<C: ConfigInterface> ConsensusMechanism<C> {
    pub fn run(vote: &VoteBuilder<C>) -> Result<ConsensusCommitment<C>> {
        let votes_by_round = VotesByIssuer::try_from(&vote.votes_by_issuer)?.into();

        let mut consensus_mechanism = Self::new(vote.committee.clone());
        consensus_mechanism.find_confirmed_milestone(votes_by_round)?;
        consensus_mechanism.find_heaviest_tip();

        Ok(ConsensusCommitment {
            confirmed_milestone: consensus_mechanism
                .accepted
                .ok_or(Error::NoAcceptedMilestoneInPastCone)?
                .into(),
            accepted_milestone: consensus_mechanism
                .confirmed
                .ok_or(Error::NoConfirmedMilestoneInPastCone)?
                .into(),
            heaviest_tip: consensus_mechanism
                .heaviest_tip
                .expect("heaviest tip should be set")
                .into(),
        })
    }

    fn new(committee: Committee<C::IssuerID>) -> Self {
        Self {
            children: HashMap::new(),
            vote_tracker: VoteTracker::new(committee.clone()),
            accepted: None,
            confirmed: None,
            heaviest_tip: None,
            committee,
        }
    }

    fn find_confirmed_milestone(&mut self, mut rounds: VotesByRound<C>) -> Result<()> {
        for round in (0..=rounds.max_round()).rev() {
            let next_targets = self.evaluate_round(rounds.fetch(round))?;
            if next_targets.is_empty() {
                break;
            }

            rounds.extend(round - 1, next_targets);
        }
        Ok(())
    }

    fn find_heaviest_tip(&mut self) {
        if let Some(mut heaviest_tip) = self.accepted.clone() {
            while let Some(heaviest_child) = self
                .children
                .get(&heaviest_tip)
                .and_then(|c| self.vote_tracker.heaviest_vote(c))
            {
                heaviest_tip = heaviest_child;
            }

            self.heaviest_tip = Some(heaviest_tip);
        }
    }

    fn evaluate_round(&mut self, votes_by_issuer: &VotesByIssuer<C>) -> Result<VotesByIssuer<C>> {
        let mut previous_round_targets = VotesByIssuer::default();
        let mut heaviest = (0, None);

        for (issuer, issuer_votes) in votes_by_issuer {
            for vote in issuer_votes {
                heaviest = max(heaviest, self.vote_tracker.track_vote(vote, issuer));

                if !vote.consensus.heaviest_tip.points_to(vote) {
                    let target = Vote::try_from(&vote.consensus.heaviest_tip)?;

                    self.children
                        .entry(target.clone())
                        .or_default()
                        .insert(vote.clone());
                    previous_round_targets.fetch(issuer.clone()).insert(target);
                }
            }
        }

        if self.accepted.is_none() && heaviest.0 >= self.committee.acceptance_threshold() {
            self.accepted = heaviest.1.clone();
        }

        if self.confirmed.is_none() && heaviest.0 >= self.committee.confirmation_threshold() {
            self.confirmed = heaviest.1.clone();

            previous_round_targets = VotesByIssuer::default();
        }

        Ok(previous_round_targets)
    }
}
