use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    sync::{Arc, Weak},
};

use crate::{ConfigInterface, Vote, VoteData};

pub struct VoteRef<T: ConfigInterface>(Weak<VoteData<T>>);

impl<T: ConfigInterface> VoteRef<T> {
    pub fn new(weak: Weak<VoteData<T>>) -> Self {
        Self(weak)
    }

    pub fn points_to(&self, vote: &Vote<T>) -> bool {
        Weak::ptr_eq(&self.0, &VoteRef::from(vote).0)
    }
}

impl<T: ConfigInterface> From<Vote<T>> for VoteRef<T> {
    fn from(vote: Vote<T>) -> Self {
        VoteRef::new(Arc::downgrade(&vote))
    }
}

impl<T: ConfigInterface> From<&Vote<T>> for VoteRef<T> {
    fn from(vote: &Vote<T>) -> Self {
        VoteRef::new(Arc::downgrade(vote))
    }
}

impl<T: ConfigInterface> Clone for VoteRef<T> {
    fn clone(&self) -> Self {
        VoteRef(self.0.clone())
    }
}

impl<T: ConfigInterface> PartialEq for VoteRef<T> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl<T: ConfigInterface> Eq for VoteRef<T> {}

impl<T: ConfigInterface> Hash for VoteRef<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.as_ptr().hash(hasher)
    }
}

impl<T: ConfigInterface> Deref for VoteRef<T> {
    type Target = Weak<VoteData<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
