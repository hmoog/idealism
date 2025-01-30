use std::collections::HashMap;

use newtype::{Default0, Deref0, FromIterator0, IntoIterator0};
use utils::ArcKey;

use crate::{ConfigInterface, VoteRef, VoteRefs, VotesByIssuer};

#[derive(Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct VoteRefsByIssuer<C: ConfigInterface>(HashMap<ArcKey<C::IssuerID>, VoteRefs<C>>);

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
