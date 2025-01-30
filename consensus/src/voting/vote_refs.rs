use std::collections::HashSet;

use newtype::{CloneInner, DefaultInner, DerefInner, FromIteratorInner, IntoIteratorInner};

use crate::{ConfigInterface, VoteRef};

#[derive(IntoIteratorInner, FromIteratorInner, DefaultInner, DerefInner, CloneInner)]
pub struct VoteRefs<Config: ConfigInterface>(HashSet<VoteRef<Config>>);
