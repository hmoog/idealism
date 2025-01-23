use std::hash::{Hash, Hasher};
use std::sync::Weak;
use crate::{ConfigInterface, Vote, VoteData};

pub struct VoteRef<T: ConfigInterface>(Weak<VoteData<T>>);

impl<T: ConfigInterface> VoteRef<T> {
    pub fn new(weak: Weak<VoteData<T>>) -> Self {
        Self(weak)
    }

    pub fn upgrade(&self) -> Option<Vote<T>> {
        self.0.upgrade().map(Vote::new)
    }

    pub fn points_to(&self, vote: &Vote<T>) -> bool {
        Weak::ptr_eq(&self.0, &vote.downgrade().0)
    }
}

impl<T: ConfigInterface> Clone for VoteRef<T> {
    fn clone(&self) -> Self {
        VoteRef(self.0.clone())
    }
}

impl<T: ConfigInterface> PartialEq for VoteRef<T> {
    fn eq (&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ConfigInterface> Eq for VoteRef<T> {}

impl<T: ConfigInterface> Hash for VoteRef<T> {
    fn hash<H : Hasher> (&self, hasher: &mut H) {
        self.0.as_ptr().hash(hasher)
    }
}