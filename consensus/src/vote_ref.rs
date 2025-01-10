use std::hash::{Hash, Hasher};
use std::sync::{Arc, Weak};
use crate::committee_member_id::CommitteeMemberID;
use crate::vote::{Vote, VoteData};

pub struct VoteRef<T: CommitteeMemberID>(Weak<VoteData<T>>);

impl<T: CommitteeMemberID> VoteRef<T> {
    pub fn upgrade(&self) -> Option<Vote<T>> {
        self.0.upgrade().map(|x| x.into())
    }
}

impl<T: CommitteeMemberID> From<&Arc<VoteData<T>>> for VoteRef<T> {
    fn from(weak: &Arc<VoteData<T>>) -> Self {
        Self(Arc::downgrade(weak))
    }
}

impl<T: CommitteeMemberID> From<&Weak<VoteData<T>>> for VoteRef<T> {
    fn from(weak: &Weak<VoteData<T>>) -> Self {
        Self(weak.clone())
    }
}

impl<T: CommitteeMemberID> Hash for VoteRef<T> {
    fn hash<H : Hasher> (self: &'_ Self, hasher: &'_ mut H) {
        self.0.as_ptr().hash(hasher)
    }
}

impl<T: CommitteeMemberID> Clone for VoteRef<T> {
    fn clone(&self) -> Self {
        VoteRef(self.0.clone())
    }
}

impl<T: CommitteeMemberID> PartialEq for VoteRef<T> {
    fn eq (self: &'_ Self, other: &'_ Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<T: CommitteeMemberID> Eq for VoteRef<T> {}