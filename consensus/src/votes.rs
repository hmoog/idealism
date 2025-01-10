use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use crate::committee_member_id::CommitteeMemberID;
use crate::vote_ref::VoteRef;

pub struct Votes<T: CommitteeMemberID>(HashSet<VoteRef<T>>);

impl<T: CommitteeMemberID> Votes<T> {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn any_round(&self) -> u64 {
        self.iter()
            .find_map(|vote_ref|
                vote_ref.upgrade().map(|vote| vote.round())
            )
            .unwrap_or(0)
    }
}

impl<T: CommitteeMemberID> IntoIterator for Votes<T> {
    type Item = VoteRef<T>;
    type IntoIter = std::collections::hash_set::IntoIter<VoteRef<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: CommitteeMemberID> Deref for Votes<T> {
    type Target = HashSet<VoteRef<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: CommitteeMemberID> DerefMut for Votes<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}