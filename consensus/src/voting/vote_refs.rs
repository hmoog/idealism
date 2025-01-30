use std::collections::HashSet;

use newtype::{Clone0, Default0, Deref0, FromIterator0, IntoIterator0};

use crate::{ConfigInterface, VoteRef};

#[derive(Clone0, Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct VoteRefs<Config: ConfigInterface>(HashSet<VoteRef<Config>>);
