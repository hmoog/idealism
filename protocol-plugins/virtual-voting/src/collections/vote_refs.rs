use std::collections::HashSet;

use zero::{Clone0, Default0, Deref0, FromIterator0, IntoIterator0};

use crate::{VirtualVotingConfig, VoteRef};

#[derive(Clone0, Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct VoteRefs<C: VirtualVotingConfig>(HashSet<VoteRef<C>>);
