use std::collections::HashMap;

use common::ids::IssuerID;
use zero::{Clone0, Default0, Deref0, FromIterator0, IntoIterator0};

use crate::{VirtualVotingConfig, Votes};

/// A collection of votes indexed by committee member ID.
///
/// This structure maintains votes from different committee members, ensuring proper handling of
/// voting rounds and vote updates.
#[derive(Default0, Deref0, IntoIterator0, FromIterator0, Clone0)]
pub struct VotesByIssuer<C: VirtualVotingConfig>(HashMap<IssuerID, Votes<C>>);

impl<C: VirtualVotingConfig> VotesByIssuer<C> {
    /// Inserts or updates votes for a committee member based on the voting round.
    ///
    /// - Clears existing votes if the new entry's round is greater.
    /// - Extends votes if the new entry's round is equal to or greater.
    ///
    /// Ensures only the most relevant votes for the latest round are retained.
    pub fn insert_or_update(&mut self, entry: Entry<C>) {
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
    pub fn fetch(&mut self, key: IssuerID) -> &mut Votes<C> {
        self.0.entry(key).or_default()
    }

    /// Checks if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

type Entry<C> = (IssuerID, Votes<C>);

mod traits {
    use super::{Entry, VotesByIssuer};
    use crate::{Error, VirtualVotingConfig, Vote, VoteRefsByIssuer, Votes};

    impl<C: VirtualVotingConfig> TryFrom<Votes<C>> for VotesByIssuer<C> {
        type Error = Error;
        fn try_from(votes: Votes<C>) -> Result<VotesByIssuer<C>, Self::Error> {
            let mut votes_by_issuer: VotesByIssuer<C> = VotesByIssuer::default();
            for vote in votes {
                votes_by_issuer.extend(VotesByIssuer::try_from(&vote.referenced_milestones)?);
            }
            Ok(votes_by_issuer)
        }
    }

    impl<C: VirtualVotingConfig> TryFrom<&VoteRefsByIssuer<C>> for VotesByIssuer<C> {
        type Error = Error;
        fn try_from(src: &VoteRefsByIssuer<C>) -> Result<VotesByIssuer<C>, Self::Error> {
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

    impl<C: VirtualVotingConfig> Extend<Entry<C>> for VotesByIssuer<C> {
        fn extend<T: IntoIterator<Item = Entry<C>>>(&mut self, entries: T) {
            for entry in entries {
                self.insert_or_update(entry);
            }
        }
    }
}
