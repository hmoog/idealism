use std::collections::{
    HashMap,
    hash_map::{IntoIter, Iter, IterMut},
};

use utils::ArcKey;

use crate::{ConfigInterface, Error, VoteRefsByIssuer, Votes};

#[derive(Default)]
pub struct VotesByIssuer<Config: ConfigInterface>(
    HashMap<ArcKey<Config::CommitteeMemberID>, Votes<Config>>,
);

impl<Config: ConfigInterface> VotesByIssuer<Config> {
    pub fn fetch(&mut self, key: ArcKey<Config::CommitteeMemberID>) -> &mut Votes<Config> {
        self.0.entry(key).or_default()
    }
}

// Implement FromIterator for collections of tuples
impl<Config: ConfigInterface> FromIterator<(ArcKey<Config::CommitteeMemberID>, Votes<Config>)>
    for VotesByIssuer<Config>
{
    fn from_iter<I: IntoIterator<Item = (ArcKey<Config::CommitteeMemberID>, Votes<Config>)>>(
        iter: I,
    ) -> Self {
        Self(iter.into_iter().collect())
    }
}

// Implement FromIterator for borrowed tuples
impl<'a, Config: ConfigInterface>
    FromIterator<(&'a ArcKey<Config::CommitteeMemberID>, &'a Votes<Config>)>
    for VotesByIssuer<Config>
{
    fn from_iter<
        I: IntoIterator<Item = (&'a ArcKey<Config::CommitteeMemberID>, &'a Votes<Config>)>,
    >(
        iter: I,
    ) -> Self {
        iter.into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl<Config: ConfigInterface> IntoIterator for VotesByIssuer<Config> {
    type Item = (ArcKey<Config::CommitteeMemberID>, Votes<Config>);
    type IntoIter = IntoIter<ArcKey<Config::CommitteeMemberID>, Votes<Config>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, Config: ConfigInterface> IntoIterator for &'a VotesByIssuer<Config> {
    type Item = (&'a ArcKey<Config::CommitteeMemberID>, &'a Votes<Config>);
    type IntoIter = Iter<'a, ArcKey<Config::CommitteeMemberID>, Votes<Config>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, Config: ConfigInterface> IntoIterator for &'a mut VotesByIssuer<Config> {
    type Item = (&'a ArcKey<Config::CommitteeMemberID>, &'a mut Votes<Config>);
    type IntoIter = IterMut<'a, ArcKey<Config::CommitteeMemberID>, Votes<Config>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<Config: ConfigInterface> TryFrom<Votes<Config>> for VotesByIssuer<Config> {
    type Error = Error;
    fn try_from(votes: Votes<Config>) -> Result<VotesByIssuer<Config>, Self::Error> {
        let mut votes_by_issuer: VotesByIssuer<Config> = VotesByIssuer::default();
        votes_by_issuer.extend(
            votes
                .into_iter()
                .map(|v| VotesByIssuer::try_from(&v.votes_by_issuer))
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(votes_by_issuer)
    }
}

impl<Config: ConfigInterface> TryFrom<VoteRefsByIssuer<Config>> for VotesByIssuer<Config> {
    type Error = Error;
    fn try_from(
        vote_refs_by_issuer: VoteRefsByIssuer<Config>,
    ) -> Result<VotesByIssuer<Config>, Self::Error> {
        vote_refs_by_issuer
            .into_inner()
            .into_iter()
            .map(|(k, v)| Votes::try_from(v).map(|v| (k, v)))
            .collect()
    }
}

impl<Config: ConfigInterface> TryFrom<&VoteRefsByIssuer<Config>> for VotesByIssuer<Config> {
    type Error = Error;
    fn try_from(src: &VoteRefsByIssuer<Config>) -> Result<VotesByIssuer<Config>, Self::Error> {
        src.into_iter()
            .map(|(k, v)| Votes::try_from(v).map(|v| (k.clone(), v)))
            .collect()
    }
}

impl<Config: ConfigInterface> Extend<VotesByIssuer<Config>> for VotesByIssuer<Config> {
    fn extend<T: IntoIterator<Item = VotesByIssuer<Config>>>(&mut self, iter: T) {
        for src in iter {
            for (issuer, src_votes) in src {
                let target_votes = self.fetch(issuer);
                let current_round = target_votes.round();
                let source_round = src_votes.round();

                if source_round > current_round {
                    target_votes.clear();
                }

                if source_round >= current_round {
                    target_votes.extend(src_votes);
                }
            }
        }
    }
}
