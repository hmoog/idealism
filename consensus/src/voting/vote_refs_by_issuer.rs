use newtype::define_hashmap;
use utils::ArcKey;

use crate::{ConfigInterface, VoteRef, VoteRefs, VotesByIssuer};

define_hashmap!(VoteRefsByIssuer, ArcKey<C::CommitteeMemberID>, VoteRefs<C>, C: ConfigInterface);

impl<C: ConfigInterface> From<VotesByIssuer<C>> for VoteRefsByIssuer<C> {
    fn from(vote: VotesByIssuer<C>) -> VoteRefsByIssuer<C> {
        vote.into_iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    VoteRefs::from_iter(v.into_iter().map(VoteRef::from)),
                )
            })
            .collect()
    }
}
