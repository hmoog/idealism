use std::hash::{Hash, Hasher};
use std::sync::{Arc, Weak};
use crate::config::ConfigInterface;
use crate::error::Error;
use crate::voting::{Vote, VoteData};

pub struct VoteRef<T: ConfigInterface>(Weak<VoteData<T>>);

impl<T: ConfigInterface> VoteRef<T> {
    pub fn upgrade(&self) -> Option<Vote<T>> {
        self.0.upgrade().map(|x| x.into())
    }

    pub fn as_vote(&self) -> Result<Vote<T>, Error> {
        self.upgrade().ok_or(Error::ReferencedVoteEvicted)
    }

    pub fn is<I: for<'a> Into<VoteRef<T>>>(&self, other: I) -> bool {
        Weak::ptr_eq(&self.0, &other.into().0)
    }
}

impl<T: ConfigInterface> TryFrom<VoteRef<T>> for Vote<T> {
    type Error = Error;

    fn try_from(value: VoteRef<T>) -> Result<Self, Self::Error> {
        value.as_vote()
    }
}

impl<T: ConfigInterface> From<Vote<T>> for VoteRef<T> {
    fn from(vote: Vote<T>) -> Self {
        vote.downgrade()
    }
}

impl<'a, T: ConfigInterface> From<&'a Vote<T>> for VoteRef<T> {
    fn from(weak: &'a Vote<T>) -> Self {
        weak.downgrade()
    }
}

impl<T: ConfigInterface> From<&Arc<VoteData<T>>> for VoteRef<T> {
    fn from(weak: &Arc<VoteData<T>>) -> Self {
        Self(Arc::downgrade(weak))
    }
}

impl<T: ConfigInterface> From<&Weak<VoteData<T>>> for VoteRef<T> {
    fn from(weak: &Weak<VoteData<T>>) -> Self {
        Self(weak.clone())
    }
}

impl<T: ConfigInterface> Hash for VoteRef<T> {
    fn hash<H : Hasher> (&self, hasher: &mut H) {
        self.0.as_ptr().hash(hasher)
    }
}

impl<T: ConfigInterface> Clone for VoteRef<T> {
    fn clone(&self) -> Self {
        VoteRef(self.0.clone())
    }
}

impl<T: ConfigInterface> PartialEq for VoteRef<T> {
    fn eq (&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<T: ConfigInterface> Eq for VoteRef<T> {}