use newtype::define_hashmap;
use utils::ArcKey;

use crate::{ConfigInterface, VoteRefsByIssuer, Votes, errors::Error};

define_hashmap!(VotesByIssuer, ArcKey<ID::CommitteeMemberID>, Votes<ID>, ID: ConfigInterface);

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
            let target_votes = self.entry(issuer.clone()).or_default();
            let current_round = target_votes.iter().next().map_or(0, |v| v.round);

            for vote in source_votes.iter() {
                if vote.round >= current_round {
                    if vote.round > current_round {
                        target_votes.clear();
                    }

                    updated = target_votes.insert(vote.clone()) || updated;
                }
            }
        }

        updated
    }
}

impl<C: ConfigInterface> TryFrom<VoteRefsByIssuer<C>> for VotesByIssuer<C> {
    type Error = Error;
    fn try_from(vote_refs_by_issuer: VoteRefsByIssuer<C>) -> Result<VotesByIssuer<C>, Self::Error> {
        vote_refs_by_issuer
            .into_inner()
            .into_iter()
            .map(|(k, v)| Votes::try_from(v).map(|v| (k, v)))
            .collect()
    }
}

impl<C: ConfigInterface> TryFrom<&VoteRefsByIssuer<C>> for VotesByIssuer<C> {
    type Error = Error;
    fn try_from(src: &VoteRefsByIssuer<C>) -> Result<VotesByIssuer<C>, Self::Error> {
        src.into_iter()
            .map(|(k, v)| Votes::try_from(v).map(|v| (k.clone(), v)))
            .collect()
    }
}
