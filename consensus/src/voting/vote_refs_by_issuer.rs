use newtype::define_hashmap;
use utils::ArcKey;

use crate::{ConfigInterface, VoteRefs};

define_hashmap!(VoteRefsByIssuer, ArcKey<C::CommitteeMemberID>, VoteRefs<C>, C: ConfigInterface);
