use std::collections::HashSet;

use crate::{ConfigInterface, Vote, VoteRefs, errors::Error};

/// A collection of votes that maintains both a set of all votes and tracks the
/// heaviest vote.
///
/// This structure provides methods for managing and querying votes, including
/// finding the heaviest vote based on provided weights.
///
/// # Type Parameters
/// * `Config`: A type that implements `ConfigInterface` which defines the
///   configuration for the voting system.
pub struct Votes<Config: ConfigInterface> {
    /// The set of all votes
    elements: HashSet<Vote<Config>>,
    /// The current heaviest vote, if any exists
    heaviest: Option<Vote<Config>>,
}

impl<Config: ConfigInterface> Votes<Config> {
    /// Inserts a new vote into the collection.
    ///
    /// Updates the heaviest vote if the new vote is greater than the current
    /// heaviest.
    ///
    /// # Returns
    /// * `true` if the vote was newly inserted
    /// * `false` if the vote was already present
    pub fn insert(&mut self, vote: Vote<Config>) -> bool {
        if self.heaviest.as_ref().map_or(true, |v| vote > *v) {
            self.heaviest = Some(vote.clone());
        }

        self.elements.insert(vote)
    }

    /// Removes all votes from the collection.
    pub fn clear(&mut self) {
        self.heaviest = None;
        self.elements.clear();
    }

    /// Returns a reference to the current heaviest vote, if one exists.
    pub fn heaviest(&self) -> &Option<Vote<Config>> {
        &self.heaviest
    }

    /// Returns the round of the current heaviest vote, or `0` if no heaviest
    /// vote exists.
    pub fn round(&self) -> u64 {
        self.heaviest.as_ref().map_or(0, |v| v.round)
    }
}

impl<Config: ConfigInterface> Default for Votes<Config> {
    fn default() -> Self {
        Self {
            elements: HashSet::new(),
            heaviest: None,
        }
    }
}

impl<Config: ConfigInterface, I: Into<Vote<Config>>> FromIterator<I> for Votes<Config> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut result = Self::default();
        result.extend(iter.into_iter().map(Into::into));
        result
    }
}

impl<Config: ConfigInterface> TryFrom<VoteRefs<Config>> for Votes<Config> {
    type Error = Error;
    fn try_from(vote_refs: VoteRefs<Config>) -> Result<Self, Self::Error> {
        vote_refs
            .into_inner()
            .into_iter()
            .map(Vote::try_from)
            .collect()
    }
}

impl<Config: ConfigInterface> TryFrom<&VoteRefs<Config>> for Votes<Config> {
    type Error = Error;
    fn try_from(vote_refs: &VoteRefs<Config>) -> Result<Self, Self::Error> {
        vote_refs.iter().map(Vote::try_from).collect()
    }
}

impl<Config: ConfigInterface> IntoIterator for Votes<Config> {
    type Item = Vote<Config>;
    type IntoIter = std::collections::hash_set::IntoIter<Vote<Config>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a, Config: ConfigInterface> IntoIterator for &'a Votes<Config> {
    type Item = &'a Vote<Config>;
    type IntoIter = std::collections::hash_set::Iter<'a, Vote<Config>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<Config: ConfigInterface> Extend<Vote<Config>> for Votes<Config> {
    fn extend<T: IntoIterator<Item = Vote<Config>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|v| {
            self.insert(v);
        });
    }
}
