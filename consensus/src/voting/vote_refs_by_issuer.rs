use std::collections::HashMap;

use newtype::{DefaultInner, DerefInner, FromIteratorInner, IntoIteratorInner};
use utils::ArcKey;

use crate::{ConfigInterface, VoteRef, VoteRefs, VotesByIssuer};

#[derive(IntoIteratorInner, FromIteratorInner, DerefInner, DefaultInner)]
pub struct VoteRefsByIssuer<C: ConfigInterface>(HashMap<ArcKey<C::CommitteeMemberID>, VoteRefs<C>>);

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
