use std::sync::Weak;

use zero::{Clone0, Default0, Deref0};

use crate::{VirtualVotingConfig, Vote, VoteBuilder};

#[derive(Clone0, Default0, Deref0)]
pub struct VoteRef<T: VirtualVotingConfig>(pub(super) Weak<VoteBuilder<T>>);

impl<C: VirtualVotingConfig> VoteRef<C> {
    pub fn points_to(&self, vote: &Vote<C>) -> bool {
        Weak::ptr_eq(&self.0, &VoteRef::from(vote).0)
    }
}

mod traits {
    use std::{
        fmt::Debug,
        hash::{Hash, Hasher},
        sync::{Arc, Weak},
    };

    use super::VoteRef;
    use crate::{VirtualVotingConfig, Vote, VoteBuilder};

    impl<C: VirtualVotingConfig> From<Weak<VoteBuilder<C>>> for VoteRef<C> {
        fn from(weak: Weak<VoteBuilder<C>>) -> Self {
            VoteRef(weak)
        }
    }

    impl<C: VirtualVotingConfig> From<&Weak<VoteBuilder<C>>> for VoteRef<C> {
        fn from(weak: &Weak<VoteBuilder<C>>) -> Self {
            VoteRef(weak.clone())
        }
    }

    impl<C: VirtualVotingConfig> From<Vote<C>> for VoteRef<C> {
        fn from(vote: Vote<C>) -> Self {
            VoteRef::from(Arc::downgrade(&vote))
        }
    }

    impl<C: VirtualVotingConfig> From<&Vote<C>> for VoteRef<C> {
        fn from(vote: &Vote<C>) -> Self {
            VoteRef::from(Arc::downgrade(vote))
        }
    }

    impl<C: VirtualVotingConfig> PartialEq for VoteRef<C> {
        fn eq(&self, other: &Self) -> bool {
            Weak::ptr_eq(&self.0, &other.0)
        }
    }

    impl<C: VirtualVotingConfig> Eq for VoteRef<C> {}

    impl<C: VirtualVotingConfig> Hash for VoteRef<C> {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            self.0.as_ptr().hash(hasher)
        }
    }

    impl<C: VirtualVotingConfig> Debug for VoteRef<C> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(builder) = self.upgrade() {
                write!(f, "VoteRef({:?}::{:?})", builder.issuer, builder.round)
            } else {
                write!(f, "VoteRef(Evicted)")
            }
        }
    }
}
