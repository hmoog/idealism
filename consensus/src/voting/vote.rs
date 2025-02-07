use std::sync::Arc;

use newtype::{Clone0, Deref0};
use utils::Id;

use crate::{ConfigInterface, Result, VoteBuilder, VoteRefs, VoteRefsByIssuer, Votes};

#[derive(Clone0, Deref0)]
pub struct Vote<Config: ConfigInterface>(Arc<VoteBuilder<Config>>);

impl<C: ConfigInterface> Vote<C> {
    pub fn new(issuer: &Id<C::IssuerID>, issuing_time: u64, latest: Vec<&Vote<C>>) -> Result<Vote<C>> {
        VoteBuilder::try_from(Votes::from_iter(latest.into_iter().cloned()))?.build(issuer.clone(), issuing_time)
    }

    pub fn from_config(config: C) -> Self {
        Self(Arc::new_cyclic(|me| {
            let mut vote = VoteBuilder::from(config);

            vote.consensus = me.into();
            vote.votes_by_issuer =
                VoteRefsByIssuer::from_iter(vote.committee.iter().map(|member| {
                    (
                        member.key().clone(),
                        VoteRefs::from_iter([vote.consensus.tip.clone()]),
                    )
                }));

            vote
        }))
    }

    pub fn consensus_weights(&self) -> (u64, u64, u64) {
        (self.cumulative_slot_weight, self.round, self.leader_weight)
    }
}

mod traits {
    use std::{
        cmp::Ordering,
        fmt::Debug,
        hash::{Hash, Hasher},
        sync::Arc,
    };

    use crate::{ConfigInterface, Error, Vote, VoteBuilder, VoteRef};

    impl<Config: ConfigInterface> From<Arc<VoteBuilder<Config>>> for Vote<Config> {
        fn from(arc: Arc<VoteBuilder<Config>>) -> Self {
            Self(arc)
        }
    }

    impl<Config: ConfigInterface> From<Config> for Vote<Config> {
        fn from(config: Config) -> Self {
            Self::from_config(config)
        }
    }

    impl<Config: ConfigInterface> TryFrom<VoteRef<Config>> for Vote<Config> {
        type Error = Error;
        fn try_from(vote_ref: VoteRef<Config>) -> Result<Self, Self::Error> {
            Vote::try_from(&vote_ref)
        }
    }

    impl<Config: ConfigInterface> TryFrom<&VoteRef<Config>> for Vote<Config> {
        type Error = Error;
        fn try_from(vote_ref: &VoteRef<Config>) -> Result<Self, Self::Error> {
            Ok(Vote::from(
                vote_ref.upgrade().ok_or(Error::ReferencedVoteEvicted)?,
            ))
        }
    }

    impl<Config: ConfigInterface> Ord for Vote<Config> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.consensus_weights().cmp(&other.consensus_weights())
        }
    }

    impl<Config: ConfigInterface> PartialOrd for Vote<Config> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<Config: ConfigInterface> Eq for Vote<Config> {}

    impl<Config: ConfigInterface> PartialEq<Self> for Vote<Config> {
        fn eq(&self, other: &Self) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }

    impl<Config: ConfigInterface> Hash for Vote<Config> {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            Arc::as_ptr(&self.0).hash(hasher)
        }
    }

    impl<Config: ConfigInterface> Debug for Vote<Config> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Vote({:?}::{:?})", self.issuer, self.round)
        }
    }
}
