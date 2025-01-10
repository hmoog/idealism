use std::cmp::max;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use utils::ArcKey;
use crate::committee_member_id::CommitteeMemberID;
use crate::votes::Votes;
use crate::votes_by_round::VotesByRound;

pub struct VotesByIssuer<ID: CommitteeMemberID>(HashMap<ArcKey<ID>, Votes<ID>>);

impl<T: CommitteeMemberID> VotesByIssuer<T> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    fn by_round(&self) -> (VotesByRound<T>, u64) {
        self.iter().fold(
            (VotesByRound::new(), 0u64),
            |(mut votes_by_round, max_round), (issuer, votes)| {
                let round = votes.any_round(); // we only use the latest votes for each issuer

                votes_by_round.insert_votes(round, issuer.clone(), votes);

                (votes_by_round, max(max_round, round))
            },
        )
    }

    fn issuers_with_round(&self, round: u64) -> usize {
        self.values().filter(|votes| votes.any_round() == round).count()
    }

    fn collect_from(&mut self, source: &VotesByIssuer<T>) -> bool {
        let mut updated = false;
        for (issuer, source_votes) in source.iter() {
            let target_votes = self.entry(issuer.clone()).or_insert_with(Votes::new);
            let current_round = target_votes.any_round();

            for vote_ref in source_votes.iter() {
                if let Some(vote) = vote_ref.upgrade() {
                    if vote.round() >= current_round {
                        if vote.round() > current_round {
                            target_votes.clear();
                        }

                        updated = target_votes.insert(vote.downgrade()) || updated;
                    }
                }
            }
        }

        updated
    }
}

impl<T: CommitteeMemberID> IntoIterator for VotesByIssuer<T> {
    type Item = (ArcKey<T>, Votes<T>);
    type IntoIter = std::collections::hash_map::IntoIter<ArcKey<T>, Votes<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: CommitteeMemberID> Deref for VotesByIssuer<T> {
    type Target = HashMap<ArcKey<T>, Votes<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: CommitteeMemberID> DerefMut for VotesByIssuer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}