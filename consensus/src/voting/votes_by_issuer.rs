use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use utils::ArcKey;

use crate::{ConfigInterface, VoteRefsByIssuer, Votes};

pub struct VotesByIssuer<ID: ConfigInterface>(HashMap<ArcKey<ID::CommitteeMemberID>, Votes<ID>>);

impl<T: ConfigInterface> VotesByIssuer<T> {
    pub fn fetch(&mut self, issuer: &ArcKey<T::CommitteeMemberID>) -> &mut Votes<T> {
        self.0.entry(issuer.clone()).or_default()
    }

    pub fn downgrade(&self) -> VoteRefsByIssuer<T> {
        self.0.iter().map(|(k, v)| (k.clone(), v.into())).collect()
    }

    pub(crate) fn collect_from(&mut self, source: &VotesByIssuer<T>) -> bool {
        let mut updated = false;
        for (issuer, source_votes) in source.iter() {
            let target_votes = self.fetch(issuer);
            let current_round = target_votes.iter().next().map_or(0, |v| v.round());

            for vote in source_votes.iter() {
                if vote.round() >= current_round {
                    if vote.round() > current_round {
                        target_votes.clear();
                    }

                    updated = target_votes.insert(vote.clone()) || updated;
                }
            }
        }

        updated
    }
}

impl<T: ConfigInterface> Default for VotesByIssuer<T> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<T: ConfigInterface> FromIterator<(ArcKey<T::CommitteeMemberID>, Votes<T>)>
    for VotesByIssuer<T>
{
    fn from_iter<I: IntoIterator<Item = (ArcKey<T::CommitteeMemberID>, Votes<T>)>>(
        iter: I,
    ) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T: ConfigInterface> Deref for VotesByIssuer<T> {
    type Target = HashMap<ArcKey<T::CommitteeMemberID>, Votes<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: ConfigInterface> DerefMut for VotesByIssuer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
