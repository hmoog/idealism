use newtype::define_hashmap;
use utils::ArcKey;

use crate::{ConfigInterface, VoteRefs, VotesByIssuer};

define_hashmap!(VoteRefsByIssuer, ArcKey<C::CommitteeMemberID>, VoteRefs<C>, C: ConfigInterface);

impl<C: ConfigInterface> From<VotesByIssuer<C>> for VoteRefsByIssuer<C> {
    fn from(vote: VotesByIssuer<C>) -> VoteRefsByIssuer<C> {
        vote.into_iter()
            .map(|(k, v)| (k.clone(), v.into()))
            .collect()
    }
}
