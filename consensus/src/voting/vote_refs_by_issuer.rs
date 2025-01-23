use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use utils::ArcKey;
use crate::errors::Error;
use crate::{ConfigInterface, VoteRefs, VotesByIssuer};

pub struct VoteRefsByIssuer<ID: ConfigInterface>(HashMap<ArcKey<ID::CommitteeMemberID>, VoteRefs<ID>>);

impl<T: ConfigInterface> VoteRefsByIssuer<T> {
    pub fn fetch(&mut self, issuer: &ArcKey<T::CommitteeMemberID>) -> &mut VoteRefs<T> {
        self.0.entry(issuer.clone()).or_default()
    }

    pub fn upgrade(&self) -> Result<VotesByIssuer<T>, Error> {
        let mut votes_by_issuer = VotesByIssuer::default();
        for (k, v) in self.0.iter() {
            votes_by_issuer.insert(k.clone(), v.upgrade()?);
        }
        Ok(votes_by_issuer)
    }
}

impl<T: ConfigInterface> Default for VoteRefsByIssuer<T> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<T: ConfigInterface> FromIterator<(ArcKey<T::CommitteeMemberID>, VoteRefs<T>)> for VoteRefsByIssuer<T> {
    fn from_iter<I: IntoIterator<Item=(ArcKey<T::CommitteeMemberID>, VoteRefs<T>)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: ConfigInterface> Deref for VoteRefsByIssuer<T> {
    type Target = HashMap<ArcKey<T::CommitteeMemberID>, VoteRefs<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ConfigInterface> DerefMut for VoteRefsByIssuer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}