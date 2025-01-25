use std::{
    hash::{Hash, Hasher},
    sync::{Arc, Weak},
};
use newtype::define;
use crate::{ConfigInterface, Vote, VoteData};

define!(VoteRef, Weak<VoteData<T>>, T: ConfigInterface);

impl<C: ConfigInterface> VoteRef<C> {
    pub fn points_to(&self, vote: &Vote<C>) -> bool {
        Weak::ptr_eq(&self.0, &VoteRef::from(vote).0)
    }
}

impl<C: ConfigInterface> From<Vote<C>> for VoteRef<C> {
    fn from(vote: Vote<C>) -> Self {
        VoteRef::new(Arc::downgrade(&vote))
    }
}

impl<C: ConfigInterface> From<&Vote<C>> for VoteRef<C> {
    fn from(vote: &Vote<C>) -> Self {
        VoteRef::new(Arc::downgrade(vote))
    }
}

impl<C: ConfigInterface> PartialEq for VoteRef<C> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl<C: ConfigInterface> Hash for VoteRef<C> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.as_ptr().hash(hasher)
    }
}
