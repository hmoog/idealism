use std::sync::Arc;

use utils::Id;
use zero::{Clone0, Deref0};

use crate::{Config, Result, VoteBuilder, Votes};

#[derive(Clone0, Deref0)]
pub struct Vote<C: Config>(Arc<VoteBuilder<C>>);

impl<C: Config> Vote<C> {
    pub fn new(issuer: &Id<C::IssuerID>, time: u64, latest: Vec<&Vote<C>>) -> Result<Vote<C>> {
        VoteBuilder::new(Votes::from_iter(latest.into_iter().cloned()))?.build(issuer, time)
    }

    pub fn from_config(config: C) -> Self {
        VoteBuilder::build_genesis(config)
    }

    pub fn weight(&self) -> (u64, u64, u64) {
        (
            self.slot_weight,
            self.round,
            self.milestone().map_or(self.time, |m| m.leader_weight),
        )
    }
}

mod traits {
    use std::{
        cmp::Ordering,
        fmt::Debug,
        hash::{Hash, Hasher},
        sync::Arc,
    };

    use crate::{Config, Error, Vote, VoteBuilder, VoteRef};

    impl<C: Config> From<Arc<VoteBuilder<C>>> for Vote<C> {
        fn from(arc: Arc<VoteBuilder<C>>) -> Self {
            Self(arc)
        }
    }

    impl<C: Config> TryFrom<VoteRef<C>> for Vote<C> {
        type Error = Error;
        fn try_from(vote_ref: VoteRef<C>) -> Result<Self, Self::Error> {
            Vote::try_from(&vote_ref)
        }
    }

    impl<C: Config> TryFrom<&VoteRef<C>> for Vote<C> {
        type Error = Error;
        fn try_from(vote_ref: &VoteRef<C>) -> Result<Self, Self::Error> {
            Ok(Vote::from(
                vote_ref.upgrade().ok_or(Error::ReferencedVoteEvicted)?,
            ))
        }
    }

    impl<C: Config> Ord for Vote<C> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.weight().cmp(&other.weight())
        }
    }

    impl<C: Config> PartialOrd for Vote<C> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<C: Config> Eq for Vote<C> {}

    impl<C: Config> PartialEq<Self> for Vote<C> {
        fn eq(&self, other: &Self) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }

    impl<C: Config> Hash for Vote<C> {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            Arc::as_ptr(&self.0).hash(hasher)
        }
    }

    impl<C: Config> Debug for Vote<C> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Vote({:?}::{:?})", self.issuer, self.round)
        }
    }
}
