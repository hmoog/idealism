use std::collections::HashMap;

use newtype::{Default0, FromIterator0, IntoIterator0};
use utils::ArcKey;

use crate::{ConfigInterface, Votes};

/// A collection of votes indexed by committee member ID.
///
/// This structure maintains votes from different committee members, ensuring proper handling of
/// voting rounds and vote updates.
#[derive(Default0, IntoIterator0, FromIterator0)]
pub struct VotesByIssuer<Config: ConfigInterface>(HashMap<ArcKey<Config::IssuerID>, Votes<Config>>);

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
    pub fn fetch(&mut self, key: ArcKey<Config::IssuerID>) -> &mut Votes<Config> {
        self.0.entry(key).or_default()
    }
}

type Entry<Config> = (ArcKey<<Config as ConfigInterface>::IssuerID>, Votes<Config>);

mod traits {
    use super::{Entry, VotesByIssuer};
    use crate::{ConfigInterface, Error, Vote, VoteRefsByIssuer, Votes};

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
        fn try_from(src: VoteRefsByIssuer<Config>) -> Result<VotesByIssuer<Config>, Self::Error> {
            Ok(VotesByIssuer(
                src.into_iter()
                    .map(|(issuer, vote_refs)| {
                        Ok((
                            issuer,
                            vote_refs
                                .into_iter()
                                .map(Vote::try_from)
                                .collect::<Result<_, _>>()?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
            ))
        }
    }

    impl<Config: ConfigInterface> TryFrom<&VoteRefsByIssuer<Config>> for VotesByIssuer<Config> {
        type Error = Error;
        fn try_from(src: &VoteRefsByIssuer<Config>) -> Result<VotesByIssuer<Config>, Self::Error> {
            Ok(VotesByIssuer(
                src.into_iter()
                    .map(|(issuer, vote_refs)| {
                        Ok((
                            issuer.clone(),
                            vote_refs
                                .iter()
                                .map(Vote::try_from)
                                .collect::<Result<_, _>>()?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
            ))
        }
    }

    impl<Config: ConfigInterface> Extend<Entry<Config>> for VotesByIssuer<Config> {
        fn extend<T: IntoIterator<Item = Entry<Config>>>(&mut self, entries: T) {
            for entry in entries {
                self.insert_or_update(entry);
            }
        }
    }
}
