use std::collections::HashMap;
use utils::ArcKey;
use crate::committee_member_id::CommitteeMemberID;
use crate::votes::Votes;
use crate::votes_by_issuer::VotesByIssuer;

pub struct VotesByRound<T: CommitteeMemberID>(HashMap<u64, VotesByIssuer<T>>);

impl<T: CommitteeMemberID> VotesByRound<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert_votes_by_issuer(&mut self, round: u64, votes_by_issuer: VotesByIssuer<T>) {
        for (issuer, votes) in votes_by_issuer {
            self.0.entry(round)
                .or_insert_with(VotesByIssuer::new)
                .entry(issuer)
                .or_insert_with(Votes::new)
                .extend(votes);
        }
    }

    pub fn insert_votes(&mut self, round: u64, issuer: ArcKey<T>, votes: &Votes<T>) {
        self.0.entry(round)
            .or_insert_with(VotesByIssuer::new)
            .entry(issuer)
            .or_insert_with(Votes::new)
            .extend(votes.iter().cloned());
    }
}

impl<T: CommitteeMemberID> IntoIterator for VotesByRound<T> {
    type Item = (u64, VotesByIssuer<T>);
    type IntoIter = std::collections::hash_map::IntoIter<u64, VotesByIssuer<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
