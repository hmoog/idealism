use std::sync::Weak;

use zero::{Clone0, Default0, Deref0};

use crate::{Config, Vote, VoteBuilder};

#[derive(Clone0, Default0, Deref0)]
pub struct VoteRef<T: Config>(pub(super) Weak<VoteBuilder<T>>);

impl<C: Config> VoteRef<C> {
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
    use crate::{Config, Vote, VoteBuilder};

    impl<C: Config> From<Weak<VoteBuilder<C>>> for VoteRef<C> {
        fn from(weak: Weak<VoteBuilder<C>>) -> Self {
            VoteRef(weak)
        }
    }

    impl<C: Config> From<&Weak<VoteBuilder<C>>> for VoteRef<C> {
        fn from(weak: &Weak<VoteBuilder<C>>) -> Self {
            VoteRef(weak.clone())
        }
    }

    impl<C: Config> From<Vote<C>> for VoteRef<C> {
        fn from(vote: Vote<C>) -> Self {
            VoteRef::from(Arc::downgrade(&vote))
        }
    }

    impl<C: Config> From<&Vote<C>> for VoteRef<C> {
        fn from(vote: &Vote<C>) -> Self {
            VoteRef::from(Arc::downgrade(vote))
        }
    }

    impl<C: Config> PartialEq for VoteRef<C> {
        fn eq(&self, other: &Self) -> bool {
            Weak::ptr_eq(&self.0, &other.0)
        }
    }

    impl<C: Config> Eq for VoteRef<C> {}

    impl<C: Config> Hash for VoteRef<C> {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            self.0.as_ptr().hash(hasher)
        }
    }

    impl<C: Config> Debug for VoteRef<C> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(builder) = self.upgrade() {
                write!(f, "VoteRef({:?}::{:?})", builder.issuer, builder.round)
            } else {
                write!(f, "VoteRef(Evicted)")
            }
        }
    }
}
