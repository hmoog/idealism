use std::collections::{HashMap, HashSet};

use utils::Id;

use crate::{Committee, ConfigInterface, Vote, Votes};

pub struct VoteTracker<C: ConfigInterface> {
    committee: Committee<C>,
    tracked_weights: HashMap<Vote<C>, u64>,
    seen_issuers: HashMap<Vote<C>, HashSet<Id<C::IssuerID>>>,
}

impl<C: ConfigInterface> VoteTracker<C> {
    pub fn new(committee: Committee<C>) -> Self {
        Self {
            committee,
            tracked_weights: HashMap::new(),
            seen_issuers: HashMap::new(),
        }
    }

    pub fn track_vote(&mut self, vote: &Vote<C>, issuer: &Id<C::IssuerID>) -> WeightEntry<C> {
        if self.issuer_voted_already(vote, issuer) {
            return (self.weight(vote), Some(vote.clone()));
        }

        let vote_weight = self.tracked_weights.entry(vote.clone()).or_insert(0);
        *vote_weight += self.committee.member_weight(issuer);

        (*vote_weight, Some(vote.clone()))
    }

    pub fn heaviest_vote(&self, votes: &Votes<C>) -> Option<Vote<C>> {
        votes
            .into_iter()
            .max_by(|a, b| (self.weight(a), a).cmp(&(self.weight(b), b)))
            .cloned()
    }

    fn weight(&self, vote: &Vote<C>) -> u64 {
        self.tracked_weights.get(vote).copied().unwrap_or(0)
    }

    fn issuer_voted_already(&mut self, vote: &Vote<C>, issuer: &Id<C::IssuerID>) -> bool {
        !self
            .seen_issuers
            .entry(vote.clone())
            .or_default()
            .insert(issuer.clone())
    }
}

pub type WeightEntry<C> = (u64, Option<Vote<C>>);
