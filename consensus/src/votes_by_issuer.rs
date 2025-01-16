use std::collections::HashMap;
use utils::ArcKey;
use crate::committee_member_id::CommitteeMemberID;
use crate::vote_refs::VoteRefs;

pub struct VotesByIssuer<ID: CommitteeMemberID>(HashMap<ArcKey<ID>, VoteRefs<ID>>);

impl<T: CommitteeMemberID> FromIterator<(ArcKey<T>, VoteRefs<T>)> for VotesByIssuer<T> {
    fn from_iter<I: IntoIterator<Item=(ArcKey<T>, VoteRefs<T>)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: CommitteeMemberID> VotesByIssuer<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, issuer: ArcKey<T>, vote: VoteRefs<T>) -> Option<VoteRefs<T>> {
        self.0.insert(issuer, vote)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<ArcKey<T>, VoteRefs<T>> {
        self.0.iter()
    }

    pub fn fetch(&mut self, issuer: &ArcKey<T>) -> &mut VoteRefs<T> {
        self.0.entry(issuer.clone()).or_insert_with(VoteRefs::default)
    }

    pub fn retain<F: FnMut(&ArcKey<T>, &mut VoteRefs<T>) -> bool>(&mut self, f: F) {
        self.0.retain(f);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn collect_from(&mut self, source: &VotesByIssuer<T>) -> bool {
        let mut updated = false;
        for (issuer, source_votes) in source {
            let target_votes = self.fetch(issuer);
            let current_round = target_votes.any_round();

            for vote_ref in source_votes {
                if let Ok(vote) = vote_ref.as_vote() {
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
    type Item = (ArcKey<T>, VoteRefs<T>);
    type IntoIter = std::collections::hash_map::IntoIter<ArcKey<T>, VoteRefs<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CommitteeMemberID> IntoIterator for &'a VotesByIssuer<T> {
    type Item = (&'a ArcKey<T>, &'a VoteRefs<T>);
    type IntoIter = std::collections::hash_map::Iter<'a, ArcKey<T>, VoteRefs<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
