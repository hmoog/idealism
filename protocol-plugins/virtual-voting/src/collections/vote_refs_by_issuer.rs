use std::collections::HashMap;

use common::{bft::Committee, ids::IssuerID};
use zero::{Default0, Deref0, FromIterator0, IntoIterator0};

use crate::{VirtualVotingConfig, VoteRef, VoteRefs, VotesByIssuer};

#[derive(Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct VoteRefsByIssuer<C: VirtualVotingConfig>(HashMap<IssuerID, VoteRefs<C>>);

impl<C: VirtualVotingConfig> VoteRefsByIssuer<C> {
    pub fn from_committee(committee: &Committee, target: &VoteRef<C>) -> VoteRefsByIssuer<C> {
        let mut vote_refs_by_issuer = VoteRefsByIssuer::default();
        for member in committee.iter() {
            vote_refs_by_issuer.insert(member.key().clone(), VoteRefs::from_iter([target.clone()]));
        }

        vote_refs_by_issuer
    }

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
    use crate::{VirtualVotingConfig, VoteRefsByIssuer, VotesByIssuer};

    impl<C: VirtualVotingConfig> From<VotesByIssuer<C>> for VoteRefsByIssuer<C> {
        fn from(src: VotesByIssuer<C>) -> VoteRefsByIssuer<C> {
            VoteRefsByIssuer::from_votes_by_issuer(&src)
        }
    }

    impl<C: VirtualVotingConfig> From<&VotesByIssuer<C>> for VoteRefsByIssuer<C> {
        fn from(src: &VotesByIssuer<C>) -> VoteRefsByIssuer<C> {
            VoteRefsByIssuer::from_votes_by_issuer(src)
        }
    }
}
