use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use crate::{ConfigInterface, VoteRef, Votes};

#[derive(Clone, Default)]
pub struct VoteRefs<C: ConfigInterface>(HashSet<VoteRef<C>>);

impl<C: ConfigInterface> From<&Votes<C>> for VoteRefs<C> {
    fn from(votes: &Votes<C>) -> Self {
        votes.iter().map(VoteRef::from).collect()
    }
}

impl<C: ConfigInterface> FromIterator<VoteRef<C>> for VoteRefs<C> {
    fn from_iter<I: IntoIterator<Item=VoteRef<C>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<C: ConfigInterface> Deref for VoteRefs<C> {
    type Target = HashSet<VoteRef<C>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: ConfigInterface> DerefMut for VoteRefs<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}