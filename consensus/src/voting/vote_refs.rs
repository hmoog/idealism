use newtype::define_hashset;
use crate::{ConfigInterface, VoteRef};

define_hashset!(VoteRefs, VoteRef<C>, C: ConfigInterface);
