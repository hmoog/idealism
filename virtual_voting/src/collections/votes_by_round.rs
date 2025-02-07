use std::{cmp::max, collections::HashMap};

use crate::{ConfigInterface, VotesByIssuer};

pub struct VotesByRound<T: ConfigInterface> {
    elements: HashMap<u64, VotesByIssuer<T>>,
    max_round: u64,
}

impl<T: ConfigInterface> VotesByRound<T> {
    pub fn from(src: VotesByIssuer<T>) -> VotesByRound<T> {
        src.into_iter().fold(
            VotesByRound::default(),
            |mut votes_by_round, (issuer, votes)| {
                votes_by_round
                    .fetch(votes.round())
                    .fetch(issuer)
                    .extend(votes);
                votes_by_round
            },
        )
    }

    pub fn extend(&mut self, round: u64, src: VotesByIssuer<T>) {
        for (issuer, votes) in src {
            self.fetch(round).fetch(issuer).extend(votes);
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

mod traits {
    use std::collections::HashMap;

    use crate::{ConfigInterface, VotesByIssuer, VotesByRound};

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
            VotesByRound::from(votes_by_issuer)
        }
    }
}
