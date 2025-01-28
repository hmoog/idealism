use std::collections::{
    HashMap,
    hash_map::{IntoIter, Iter},
};

use utils::ArcKey;

use crate::{ConfigInterface, Error, VoteRefsByIssuer, Votes};

/// A collection of votes indexed by committee member ID.
///
/// This structure maintains votes from different committee members, ensuring proper
/// handling of voting rounds and vote updates.
pub struct VotesByIssuer<Config: ConfigInterface> {
    elements: HashMap<ArcKey<Config::CommitteeMemberID>, Votes<Config>>,
}

impl<Config: ConfigInterface> VotesByIssuer<Config> {
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

/// Implements vote collection extension with round-based conflict resolution.
///
/// When extending the collection with new votes:
/// 1. If the source round is higher than the current round, clear existing votes
/// 2. Only extend with votes from equal or higher rounds
impl<Config: ConfigInterface> Extend<Entry<Config>> for VotesByIssuer<Config> {
    fn extend<T: IntoIterator<Item = Entry<Config>>>(&mut self, iter: T) {
        for (issuer, src_votes) in iter.into_iter() {
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

/// A tuple of committee member ID and their votes.
type Entry<Config> = (
    ArcKey<<Config as ConfigInterface>::CommitteeMemberID>,
    Votes<Config>,
);

/// A reference to an Entry.
type EntryRef<'a, Config> = (
    &'a ArcKey<<Config as ConfigInterface>::CommitteeMemberID>,
    &'a Votes<Config>,
);
