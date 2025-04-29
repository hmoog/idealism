use common::collections::MaxSet;
use zero::{Clone0, Default0, Deref0, FromIterator0, IntoIterator0};

use crate::{VirtualVotingConfig, Vote};

#[derive(Clone0, Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct Votes<C: VirtualVotingConfig>(MaxSet<Vote<C>>);

impl<C: VirtualVotingConfig> Votes<C> {
    pub fn round(&self) -> u64 {
        self.heaviest_element().map_or(0, |v| v.round)
    }
}
