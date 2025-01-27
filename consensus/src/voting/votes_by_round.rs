use std::{cmp::max, collections::HashMap};

use crate::{ConfigInterface, Vote, VotesByIssuer};

pub struct VotesByRound<T: ConfigInterface> {
    elements: HashMap<u64, VotesByIssuer<T>>,
    max_round: u64,
}

impl<T: ConfigInterface> VotesByRound<T> {
    pub fn insert_votes_by_issuer(&mut self, round: u64, votes_by_issuer: VotesByIssuer<T>) {
        for (issuer, votes) in votes_by_issuer {
            self.fetch(round).entry(issuer).or_default().extend(votes);
        }
    }

    pub fn fetch(&mut self, round: u64) -> &mut VotesByIssuer<T> {
        self.max_round = max(self.max_round, round);
        self.elements.entry(round).or_default()
    }

    pub fn max_round(&self) -> u64 {
        self.max_round
    }
}

impl<T: ConfigInterface> Default for VotesByRound<T> {
    fn default() -> Self {
        Self {
            elements: HashMap::new(),
            max_round: 0,
        }
    }
}

impl<T: ConfigInterface> From<VotesByIssuer<T>> for VotesByRound<T> {
    fn from(votes_by_issuer: VotesByIssuer<T>) -> VotesByRound<T> {
        votes_by_issuer.into_inner().into_iter().fold(
            VotesByRound::default(),
            |mut votes_by_round, (issuer, votes)| {
                votes_by_round
                    .fetch(votes.round())
                    .entry(issuer)
                    .or_default()
                    .extend(votes);
                votes_by_round
            },
        )
    }
}

impl<T: ConfigInterface> From<&VotesByIssuer<T>> for VotesByRound<T> {
    fn from(votes_by_issuer: &VotesByIssuer<T>) -> VotesByRound<T> {
        votes_by_issuer.iter().fold(
            VotesByRound::default(),
            |mut votes_by_round, (issuer, votes)| {
                votes_by_round
                    .fetch(votes.round())
                    .entry(issuer.clone())
                    .or_default()
                    .extend(votes.into_iter().map(Vote::clone));
                votes_by_round
            },
        )
    }
}
