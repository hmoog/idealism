use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use crate::errors::Error;
use crate::{ConfigInterface, VoteRef, Votes};

#[derive(Clone, Default)]
pub struct VoteRefs<T: ConfigInterface>(HashSet<VoteRef<T>>);

impl<T: ConfigInterface> VoteRefs<T> {
    pub fn upgrade(&self) -> Result<Votes<T>, Error> {
        self.iter()
            .map(|vote_ref| vote_ref.upgrade().ok_or(Error::ReferencedVoteEvicted))
            .collect()
    }
}

impl<ID: ConfigInterface> FromIterator<VoteRef<ID>> for VoteRefs<ID> {
    fn from_iter<I: IntoIterator<Item=VoteRef<ID>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: ConfigInterface> Deref for VoteRefs<T> {
    type Target = HashSet<VoteRef<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ConfigInterface> DerefMut for VoteRefs<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}