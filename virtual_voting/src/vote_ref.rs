use std::sync::Weak;

use zero::{Clone0, Default0, Deref0};

use crate::{ConfigInterface, Vote, VoteBuilder};

#[derive(Clone0, Default0, Deref0)]
pub struct VoteRef<T: ConfigInterface>(pub(super) Weak<VoteBuilder<T>>);

impl<C: ConfigInterface> VoteRef<C> {
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
    use crate::{ConfigInterface, Vote, VoteBuilder};

    impl<C: ConfigInterface> From<Weak<VoteBuilder<C>>> for VoteRef<C> {
        fn from(weak: Weak<VoteBuilder<C>>) -> Self {
            VoteRef(weak)
        }
    }

    impl<C: ConfigInterface> From<&Weak<VoteBuilder<C>>> for VoteRef<C> {
        fn from(weak: &Weak<VoteBuilder<C>>) -> Self {
            VoteRef(weak.clone())
        }
    }

    impl<C: ConfigInterface> From<Vote<C>> for VoteRef<C> {
        fn from(vote: Vote<C>) -> Self {
            VoteRef::from(Arc::downgrade(&vote))
        }
    }

    impl<C: ConfigInterface> From<&Vote<C>> for VoteRef<C> {
        fn from(vote: &Vote<C>) -> Self {
            VoteRef::from(Arc::downgrade(vote))
        }
    }

    impl<C: ConfigInterface> PartialEq for VoteRef<C> {
        fn eq(&self, other: &Self) -> bool {
            Weak::ptr_eq(&self.0, &other.0)
        }
    }

    impl<C: ConfigInterface> Eq for VoteRef<C> {}

    impl<C: ConfigInterface> Hash for VoteRef<C> {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            self.0.as_ptr().hash(hasher)
        }
    }

    impl<Config: ConfigInterface> Debug for VoteRef<Config> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if let Some(builder) = self.upgrade() {
                write!(f, "VoteRef({:?}::{:?})", builder.issuer, builder.round)
            } else {
                write!(f, "VoteRef(Evicted)")
            }
        }
    }
}
