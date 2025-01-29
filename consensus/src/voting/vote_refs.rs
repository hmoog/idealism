use std::collections::HashSet;

use newtypev1::{CloneInner, DefaultInner, DerefInner, FromIteratorInner, IntoIteratorInner};

use crate::{ConfigInterface, VoteRef};

#[derive(IntoIteratorInner, FromIteratorInner, DefaultInner, DerefInner, CloneInner)]
pub struct VoteRefs<Config: ConfigInterface>(HashSet<VoteRef<Config>>);
