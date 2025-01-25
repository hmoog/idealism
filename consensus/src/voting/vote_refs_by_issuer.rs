use newtype::define_hashmap;
use utils::ArcKey;

use crate::{ConfigInterface, VoteRefs};

define_hashmap!(VoteRefsByIssuer, ArcKey<C::CommitteeMemberID>, VoteRefs<C>, C: ConfigInterface);

impl<C: ConfigInterface> VoteRefsByIssuer<C> {
    pub fn fetch(&mut self, issuer: &ArcKey<C::CommitteeMemberID>) -> &mut VoteRefs<C> {
        self.0.entry(issuer.clone()).or_default()
    }
}
