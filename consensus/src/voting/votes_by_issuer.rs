use newtype::define_hashmap;
use utils::ArcKey;

use crate::{ConfigInterface, VoteRefsByIssuer, Votes, errors::Error};

define_hashmap!(VotesByIssuer, ArcKey<ID::CommitteeMemberID>, Votes<ID>, ID: ConfigInterface);

impl<T: ConfigInterface> VotesByIssuer<T> {
    pub fn downgrade(&self) -> VoteRefsByIssuer<T> {
        self.0.iter().map(|(k, v)| (k.clone(), v.into())).collect()
    }
}

impl<C: ConfigInterface> TryFrom<Votes<C>> for VotesByIssuer<C> {
    type Error = Error;
    fn try_from(votes: Votes<C>) -> Result<VotesByIssuer<C>, Self::Error> {
        let mut votes_by_issuer: VotesByIssuer<C> = VotesByIssuer::default();
        votes_by_issuer.extend(
            votes
                .into_iter()
                .map(|v| VotesByIssuer::try_from(&v.votes_by_issuer))
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(votes_by_issuer)
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

impl<Config: ConfigInterface> Extend<VotesByIssuer<Config>> for VotesByIssuer<Config> {
    fn extend<T: IntoIterator<Item = VotesByIssuer<Config>>>(&mut self, iter: T) {
        for src in iter {
            for (issuer, src_votes) in src {
                let target_votes = self.entry(issuer).or_default();
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
