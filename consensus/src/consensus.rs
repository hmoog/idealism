use std::cmp::max;
use std::collections::{HashMap};
use crate::committee::Committee;
use crate::committee_member_id::CommitteeMemberID;
use crate::consensus::WalkResult::{LatestAcceptedMilestoneFound, PreviousRoundTargets};
use crate::vote::Vote;
use crate::vote_ref::VoteRef;
use crate::vote_refs::VoteRefs;
use crate::votes_by_issuer::VotesByIssuer;
use crate::votes_by_round::VotesByRound;

pub(crate) struct ConsensusRound<ID: CommitteeMemberID> {
    committee: Committee<ID>,
    children: HashMap<VoteRef<ID>, VoteRefs<ID>>,
    weights: HashMap<VoteRef<ID>, u64>,
}

impl<ID: CommitteeMemberID> ConsensusRound<ID> {
    pub(crate) fn new(committee: Committee<ID>) -> Self {
        Self {
            committee,
            children: HashMap::new(),
            weights: HashMap::new(),
        }
    }

    /// Walks through the votes of each round and returns the latest accepted milestone.
    pub(crate) fn latest_accepted_milestone(&mut self, mut votes_by_round: VotesByRound<ID>) -> VoteRef<ID> {
        for round in (0..=votes_by_round.max_round()).rev() {
            match self.heaviest_target(votes_by_round.fetch(round)) {
                LatestAcceptedMilestoneFound(latest_accepted_milestone) =>
                    return latest_accepted_milestone.downgrade(),
                PreviousRoundTargets(previous_round_targets) => if round > 0 {
                    votes_by_round.insert_votes_by_issuer(round - 1, previous_round_targets)
                },
            }
        }

        unreachable!("we should never reach this point in the logic as the root is always accepted")
    }

    pub(crate) fn heaviest_descendant(&self, vote: &VoteRef<ID>) -> VoteRef<ID> {
        let mut heaviest_descendant = vote.clone();
        while let Some(children) = self.children.get(&heaviest_descendant) {
            match self.heaviest_vote(children) {
                Some(heaviest_child) => heaviest_descendant = heaviest_child,
                None => break,
            }
        }

        heaviest_descendant
    }

    fn add_weight(&mut self, vote: &VoteRef<ID>, weight: u64) -> u64 {
        *self.weights.entry(vote.clone()).and_modify(|w| *w += weight).or_insert(weight)
    }

    fn heaviest_target(&mut self, votes_of_round: &VotesByIssuer<ID>) -> WalkResult<ID> {
        let mut targets = VotesByIssuer::new();
        let mut heaviest_vote = None;
        let mut heaviest_weight = 0;

        for (issuer, votes) in votes_of_round {
            for vote in votes {
                let Ok(vote) = vote.as_vote() else { continue };
                let Ok(target) = vote.target().as_vote() else { continue };

                if vote.is_accepted() {
                    return LatestAcceptedMilestoneFound(vote);
                }

                let issuer_weight = self.committee.member_weight(vote.issuer());
                let updated_weight = self.add_weight(&vote.target(), issuer_weight);

                if updated_weight > heaviest_weight {
                    heaviest_vote = Some(target);
                    heaviest_weight = updated_weight;
                } else if updated_weight == heaviest_weight {
                    heaviest_vote = max(heaviest_vote, Some(target));
                }

                let targets = targets.fetch(issuer);
                targets.insert(vote.target().clone());

                let children = self.children.entry(vote.target().clone()).or_insert_with(VoteRefs::default);
                children.insert(vote.downgrade());
            }
        }

        match heaviest_weight > (2.0 / 3.0 * self.committee.online_weight() as f64) as u64 {
            true => LatestAcceptedMilestoneFound(heaviest_vote.unwrap()),
            false => PreviousRoundTargets(targets),
        }
    }

    fn heaviest_vote(&self, votes: &VoteRefs<ID>) -> Option<VoteRef<ID>> {
        votes.iter()
            .filter_map(|candidate_weak| {
                Some((candidate_weak.upgrade()?, self.weights.get(candidate_weak).unwrap_or(&0)))
            })
            .max_by(|(candidate1, weight1), (candidate2, weight2)| {
                weight1.cmp(weight2).then_with(|| candidate1.cmp(candidate2))
            })
            .map(|(candidate, _)| { candidate.downgrade() })
    }
}

enum WalkResult<ID: CommitteeMemberID> {
    LatestAcceptedMilestoneFound(Vote<ID>),
    PreviousRoundTargets(VotesByIssuer<ID>),
}
