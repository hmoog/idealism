use std::collections::HashSet;

use zero::{Clone0, Default0, Deref0, FromIterator0, IntoIterator0};

use crate::{Config, VoteRef};

#[derive(Clone0, Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct VoteRefs<C: Config>(HashSet<VoteRef<C>>);
