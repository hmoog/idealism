use std::cmp::max;
use std::collections::HashMap;
use crate::committee_member_id::CommitteeMemberID;
use crate::votes_by_issuer::VotesByIssuer;

pub struct VotesByRound<T: CommitteeMemberID> {
    elements: HashMap<u64, VotesByIssuer<T>>,
    max_round: u64,
}

impl<T: CommitteeMemberID> VotesByRound<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_votes_by_issuer(&mut self, round: u64, votes_by_issuer: VotesByIssuer<T>) {
        for (issuer, votes) in votes_by_issuer {
            self.fetch(round).fetch(&issuer).extend(votes);
        }
    }

    pub fn fetch(&mut self, round: u64) -> &mut VotesByIssuer<T> {
        self.max_round = max(self.max_round, round);
        self.elements.entry(round).or_insert_with(VotesByIssuer::new)
    }

    pub fn max_round(&self) -> u64 {
        self.max_round
    }
}

impl<T: CommitteeMemberID> Default for VotesByRound<T> {
    fn default() -> Self {
        Self {
            elements: HashMap::new(),
            max_round: 0,
        }
    }
}

impl<T: CommitteeMemberID> From<&VotesByIssuer<T>> for VotesByRound<T> {
    fn from(votes_by_issuer: &VotesByIssuer<T>) -> VotesByRound<T> {
        votes_by_issuer.iter().fold(VotesByRound::new(), |mut votes_by_round, (issuer, votes)| {
            votes_by_round
                .fetch(votes.any_round())
                .fetch(issuer)
                .extend(votes.clone());
            votes_by_round
        })
    }
}

impl<T: CommitteeMemberID> IntoIterator for VotesByRound<T> {
    type Item = (u64, VotesByIssuer<T>);
    type IntoIter = std::collections::hash_map::IntoIter<u64, VotesByIssuer<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}
