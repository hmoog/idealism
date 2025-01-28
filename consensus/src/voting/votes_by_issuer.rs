use std::collections::{
    HashMap,
    hash_map::{IntoIter, Iter},
};

use utils::ArcKey;

use crate::{ConfigInterface, Error, VoteRefsByIssuer, Votes};

/// A collection of votes indexed by committee member ID.
///
/// This structure maintains votes from different committee members, ensuring proper handling of
/// voting rounds and vote updates.
pub struct VotesByIssuer<Config: ConfigInterface> {
    elements: HashMap<ArcKey<Config::CommitteeMemberID>, Votes<Config>>,
}

impl<Config: ConfigInterface> VotesByIssuer<Config> {
    /// Inserts or updates votes for a committee member based on the voting round.
    ///
    /// - Clears existing votes if the new entry's round is greater.
    /// - Extends votes if the new entry's round is equal to or greater.
    ///
    /// Ensures only the most relevant votes for the latest round are retained.
    pub fn insert_or_update(&mut self, entry: Entry<Config>) {
        let target_votes = self.fetch(entry.0);
        let current_round = target_votes.round();
        let new_round = entry.1.round();

        if new_round > current_round {
            target_votes.clear();
        }

        if new_round >= current_round {
            target_votes.extend(entry.1);
        }
    }

    /// Retrieves or creates a mutable reference to the votes for a given committee member.
    ///
    /// If no votes exist for the given key, a new empty votes collection is created.
    pub fn fetch(&mut self, key: ArcKey<Config::CommitteeMemberID>) -> &mut Votes<Config> {
        self.elements.entry(key).or_default()
    }
}

impl<Config: ConfigInterface> Default for VotesByIssuer<Config> {
    fn default() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }
}

impl<Config: ConfigInterface> TryFrom<Votes<Config>> for VotesByIssuer<Config> {
    type Error = Error;
    fn try_from(votes: Votes<Config>) -> Result<VotesByIssuer<Config>, Self::Error> {
        let mut votes_by_issuer: VotesByIssuer<Config> = VotesByIssuer::default();
        for vote in votes {
            votes_by_issuer.extend(VotesByIssuer::try_from(&vote.votes_by_issuer)?);
        }
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

impl<Config: ConfigInterface> FromIterator<Entry<Config>> for VotesByIssuer<Config> {
    fn from_iter<I: IntoIterator<Item = Entry<Config>>>(iter: I) -> Self {
        Self {
            elements: iter.into_iter().collect(),
        }
    }
}

impl<'a, Config: ConfigInterface> FromIterator<EntryRef<'a, Config>> for VotesByIssuer<Config> {
    fn from_iter<I: IntoIterator<Item = EntryRef<'a, Config>>>(iter: I) -> Self {
        iter.into_iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl<Config: ConfigInterface> IntoIterator for VotesByIssuer<Config> {
    type Item = Entry<Config>;
    type IntoIter = IntoIter<ArcKey<Config::CommitteeMemberID>, Votes<Config>>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a, Config: ConfigInterface> IntoIterator for &'a VotesByIssuer<Config> {
    type Item = EntryRef<'a, Config>;
    type IntoIter = Iter<'a, ArcKey<Config::CommitteeMemberID>, Votes<Config>>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<Config: ConfigInterface> Extend<Entry<Config>> for VotesByIssuer<Config> {
    fn extend<T: IntoIterator<Item = Entry<Config>>>(&mut self, entries: T) {
        for entry in entries {
            self.insert_or_update(entry);
        }
    }
}

type Entry<Config> = (
    ArcKey<<Config as ConfigInterface>::CommitteeMemberID>,
    Votes<Config>,
);

type EntryRef<'a, Config> = (
    &'a ArcKey<<Config as ConfigInterface>::CommitteeMemberID>,
    &'a Votes<Config>,
);
