use std::collections::HashMap;
use utils::ArcKey;
use crate::error::Error;
use crate::{ConfigInterface, VoteRefs, VotesByIssuer};

pub struct VoteRefsByIssuer<ID: ConfigInterface>(HashMap<ArcKey<ID::CommitteeMemberID>, VoteRefs<ID>>);

impl<T: ConfigInterface> FromIterator<(ArcKey<T::CommitteeMemberID>, VoteRefs<T>)> for VoteRefsByIssuer<T> {
    fn from_iter<I: IntoIterator<Item=(ArcKey<T::CommitteeMemberID>, VoteRefs<T>)>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: ConfigInterface> VoteRefsByIssuer<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, issuer: ArcKey<T::CommitteeMemberID>, vote: VoteRefs<T>) -> Option<VoteRefs<T>> {
        self.0.insert(issuer, vote)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<ArcKey<T::CommitteeMemberID>, VoteRefs<T>> {
        self.0.iter()
    }

    pub fn fetch(&mut self, issuer: &ArcKey<T::CommitteeMemberID>) -> &mut VoteRefs<T> {
        self.0.entry(issuer.clone()).or_default()
    }

    pub fn retain<F: FnMut(&ArcKey<T::CommitteeMemberID>, &mut VoteRefs<T>) -> bool>(&mut self, f: F) {
        self.0.retain(f);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn upgrade(&self) -> Result<VotesByIssuer<T>, Error> {
        let mut votes_by_issuer = VotesByIssuer::new();
        for (k, v) in self.0.iter() {
            votes_by_issuer.insert(k.clone(), v.upgrade()?);
        }
        Ok(votes_by_issuer)
    }
}

impl<T: ConfigInterface> Default for VoteRefsByIssuer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ConfigInterface> IntoIterator for VoteRefsByIssuer<T> {
    type Item = (ArcKey<T::CommitteeMemberID>, VoteRefs<T>);
    type IntoIter = std::collections::hash_map::IntoIter<ArcKey<T::CommitteeMemberID>, VoteRefs<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: ConfigInterface> IntoIterator for &'a VoteRefsByIssuer<T> {
    type Item = (&'a ArcKey<T::CommitteeMemberID>, &'a VoteRefs<T>);
    type IntoIter = std::collections::hash_map::Iter<'a, ArcKey<T::CommitteeMemberID>, VoteRefs<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
