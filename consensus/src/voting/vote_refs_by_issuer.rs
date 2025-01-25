use newtype::define_hashmap;
use utils::ArcKey;

use crate::{ConfigInterface, VoteRefs, VotesByIssuer, errors::Error};

define_hashmap!(VoteRefsByIssuer, ArcKey<C::CommitteeMemberID>, VoteRefs<C>, C: ConfigInterface);

impl<C: ConfigInterface> VoteRefsByIssuer<C> {
    pub fn fetch(&mut self, issuer: &ArcKey<C::CommitteeMemberID>) -> &mut VoteRefs<C> {
        self.0.entry(issuer.clone()).or_default()
    }

    pub fn upgrade(&self) -> Result<VotesByIssuer<C>, Error> {
        let mut votes_by_issuer = VotesByIssuer::default();
        for (k, v) in self.0.iter() {
            votes_by_issuer.insert(k.clone(), v.try_into()?);
        }
        Ok(votes_by_issuer)
    }
}
