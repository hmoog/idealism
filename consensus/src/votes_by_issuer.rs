use std::collections::HashMap;
use utils::ArcKey;
use crate::committee_member_id::CommitteeMemberID;
use crate::votes::Votes;

pub struct VotesByIssuer<ID: CommitteeMemberID>(HashMap<ArcKey<ID>, Votes<ID>>);

impl<T: CommitteeMemberID> VotesByIssuer<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<ArcKey<T>, Votes<T>> {
        self.0.iter()
    }

    pub fn fetch(&mut self, issuer: &ArcKey<T>) -> &mut Votes<T> {
        self.0.entry(issuer.clone()).or_insert_with(Votes::new)
    }

    fn issuers_with_round(&self, round: u64) -> usize {
        self.0.values().filter(|votes| votes.any_round() == round).count()
    }

    fn collect_from(&mut self, source: &VotesByIssuer<T>) -> bool {
        let mut updated = false;
        for (issuer, source_votes) in source {
            let target_votes = self.fetch(issuer);
            let current_round = target_votes.any_round();

            for vote_ref in source_votes.iter() {
                if let Some(vote) = vote_ref.upgrade() {
                    if vote.round() >= current_round {
                        if vote.round() > current_round {
                            target_votes.clear();
                        }

                        updated = target_votes.insert(vote.downgrade()) || updated;
                    }
                }
            }
        }

        updated
    }
}

impl<T: CommitteeMemberID> IntoIterator for VotesByIssuer<T> {
    type Item = (ArcKey<T>, Votes<T>);
    type IntoIter = std::collections::hash_map::IntoIter<ArcKey<T>, Votes<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CommitteeMemberID> IntoIterator for &'a VotesByIssuer<T> {
    type Item = (&'a ArcKey<T>, &'a Votes<T>);
    type IntoIter = std::collections::hash_map::Iter<'a, ArcKey<T>, Votes<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
