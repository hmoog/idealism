use std::collections::HashMap;

use utils::Id;
use zero::{Default0, Deref0, FromIterator0, IntoIterator0};

use crate::{Config, VoteRefs, VotesByIssuer};

#[derive(Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct VoteRefsByIssuer<C: Config>(HashMap<Id<C::IssuerID>, VoteRefs<C>>);

impl<C: Config> VoteRefsByIssuer<C> {
    pub fn from_votes_by_issuer(vote: &VotesByIssuer<C>) -> VoteRefsByIssuer<C> {
        vote.into_iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    VoteRefs::from_iter(v.into_iter().map(Into::into)),
                )
            })
            .collect()
    }
}

mod traits {
    use crate::{Config, VoteRefsByIssuer, VotesByIssuer};

    impl<C: Config> From<VotesByIssuer<C>> for VoteRefsByIssuer<C> {
        fn from(src: VotesByIssuer<C>) -> VoteRefsByIssuer<C> {
            VoteRefsByIssuer::from_votes_by_issuer(&src)
        }
    }

    impl<C: Config> From<&VotesByIssuer<C>> for VoteRefsByIssuer<C> {
        fn from(src: &VotesByIssuer<C>) -> VoteRefsByIssuer<C> {
            VoteRefsByIssuer::from_votes_by_issuer(src)
        }
    }
}
