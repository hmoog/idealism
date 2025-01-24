use newtype::define_hashset;
use crate::{ConfigInterface, VoteRef, Votes};

define_hashset!(VoteRefs, VoteRef<C>, C: ConfigInterface);

impl<C: ConfigInterface> From<Votes<C>> for VoteRefs<C> {
    fn from(votes: Votes<C>) -> VoteRefs<C> {
        votes.into_iter().map(VoteRef::from).collect()
    }
}

impl<C: ConfigInterface> From<&Votes<C>> for VoteRefs<C> {
    fn from(votes: &Votes<C>) -> VoteRefs<C> {
        votes.iter().map(VoteRef::from).collect()
    }
}
